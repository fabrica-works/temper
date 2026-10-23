"""Narrow delegated-unit bootstrap; all admission and locking are in its worker.

Unlike runner.transport, no runner worker is invoked and no product output is
connected to this transport. The worker owns its lab lock for the whole run.
"""
import json
import os
import select
import signal
import subprocess
import sys
import time
from _host_output import Output


def launch(args, native, store, host_digest, runner_digest):
    unit = 'lab-attended-' + str(os.getpid()) + '-' + str(time.monotonic_ns()) + '.service'
    command = ['/usr/bin/systemd-run', '--user', '--no-ask-password', '--quiet', '--wait',
               '--pipe', '--collect', '--expand-environment=no', '--service-type=exec',
               '--unit=' + unit, '--property=Delegate=yes', '--property=KillMode=control-group',
               '--property=TimeoutStartSec=120', '--property=TimeoutStopSec=20', '--property=LimitCORE=0',
               '/usr/bin/env', '-i', 'PATH=/usr/bin:/bin', os.path.realpath(sys.executable),
               '-I', '-B', '-S', str(args.host_file), '--skein', str(args.skein),
               '--lab', str(args.lab), '--experiment', str(args.experiment),
               '--subject', args.subject, '--seed', args.seed, '--unit', unit,
               '--expected-host', host_digest, '--expected-runner', runner_digest]
    wake_r, wake_w = os.pipe2(os.O_CLOEXEC | os.O_NONBLOCK)
    saved = {sig: signal.getsignal(sig) for sig in (signal.SIGINT, signal.SIGTERM)}
    received = []
    for sig in saved:
        signal.signal(sig, lambda sig, _frame: received.append(sig))
    old_wake = signal.set_wakeup_fd(wake_w)
    process = None
    pidfd = None
    buffers = {1: bytearray(), 2: bytearray()}
    completed = False
    status, cause = None, 'unit not completed'
    admitted = False
    admission_deadline = time.monotonic() + 120
    try:
        process = subprocess.Popen(command, env=native.manager_environment(), stdin=subprocess.DEVNULL,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        pidfd = os.pidfd_open(process.pid)
        poller = select.poll()
        poller.register(wake_r, select.POLLIN)
        poller.register(pidfd, select.POLLIN)
        streams = {process.stdout.fileno(): 1, process.stderr.fileno(): 2}
        for fd in streams:
            poller.register(fd, select.POLLIN | select.POLLHUP)
        while streams or process.poll() is None:
            remaining = None if admitted else admission_deadline - time.monotonic()
            if remaining is not None and remaining <= 0:
                raise TimeoutError('unit admission exceeded ready_seconds')
            timeout = None if remaining is None else max(1, int(remaining * 1000))
            for fd, _event in poller.poll(timeout):
                if fd == wake_r:
                    os.read(wake_r, 4096)
                    while received:
                        sig = received.pop(0)
                        native.manager('/usr/bin/systemctl', ['kill', '--kill-whom=main',
                                       '--signal=' + signal.Signals(sig).name, unit], 240)
                elif fd == pidfd:
                    poller.unregister(pidfd)
                    process.wait()
                elif fd in streams:
                    chunk = os.read(fd, 65536)
                    if chunk:
                        buffers[streams[fd]].extend(chunk)
                        admitted = bytes(buffers[1]).startswith(b'admitted\n')
                    else:
                        poller.unregister(fd)
                        del streams[fd]
        status = process.wait()
        # Unit diagnostics contain host/manager output only, and still pass
        # through the credential filter before their write-once publication.
        with store.open_lab(args.lab) as lab, lab.lock():
            for number, name in ((1, 'stdout'), (2, 'stderr')):
                framed = Output()
                records = framed.feed(bytes(buffers[number]), eof=True)
                lab.write_once('transport/' + unit + '/' + name, b''.join(r[0] for r in records))
        # Worker stdout has one nonsecret JSON summary. Never expose arbitrary
        # manager diagnostics/tracebacks to the attended terminal.
        try:
            summary = json.loads(bytes(buffers[1]).removeprefix(b'admitted\n'))
            if set(summary) != {'run', 'outcome', 'cgroup'}:
                raise ValueError
            if summary['cgroup']:
                native.await_removed(summary['cgroup'], 20)
            completed = True
            cause = 'unit exited and cgroup removal confirmed'
            print(summary['run'] + ' ' + summary['outcome'])
        except (ValueError, KeyError, TypeError):
            try:
                native.manager('/usr/bin/systemctl', ['kill', '--kill-whom=all', '--signal=SIGKILL', unit], 240)
            except native.CapabilityError:
                pass
            status = 4
            cause = 'unit ended without a valid summary'
            print('attended host: unit ended without a summary; inspect retained lab evidence', file=sys.stderr)
        return status
    except BaseException as error:
        cause = type(error).__name__
        raise
    finally:
        if process is not None and not completed:
            try:
                native.manager('/usr/bin/systemctl', ['kill', '--kill-whom=all', '--signal=SIGKILL', unit], 240)
            except native.CapabilityError:
                pass
            if process.poll() is None:
                # The owned launcher must not outlive an aborted bootstrap.
                signal.pidfd_send_signal(pidfd, signal.SIGKILL)
                process.wait(timeout=20)
        # Retain a cause even if native unit completion failed after run.json
        # was published. This transport result never edits prior evidence.
        try:
            with store.open_lab(args.lab) as lab, lab.lock():
                result = f'exit={status if completed and status is not None else 4}\ncleanup_confirmed={completed}\ncause={cause}\n'
                lab.write_once('transport/' + unit + '/result', result.encode('utf-8'))
        except (OSError, RuntimeError, ValueError):
            print('attended host: transport result publication unavailable', file=sys.stderr)
        if pidfd is not None:
            os.close(pidfd)
        if process is not None:
            process.stdout.close()
            process.stderr.close()
        signal.set_wakeup_fd(old_wake)
        for sig, handler in saved.items():
            signal.signal(sig, handler)
        os.close(wake_r)
        os.close(wake_w)
