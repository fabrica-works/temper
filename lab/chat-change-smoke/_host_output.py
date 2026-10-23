"""Line framing before persistence; credentials never leave this memory boundary."""
import base64
import ipaddress
import re
from urllib.parse import unquote_to_bytes, urlsplit


def origin(url):
    """Require an exact numeric loopback HTTP origin with an explicit port."""
    try:
        parts = urlsplit(url)
        if (parts.scheme != 'http' or parts.username is not None or parts.password is not None
                or parts.port is None or not 1 <= parts.port <= 65535
                or '%' in (parts.hostname or '')
                or not ipaddress.ip_address(parts.hostname).is_loopback):
            raise ValueError
        host = parts.hostname
        if ':' in host:
            host = '[' + host + ']'
        public = f'http://{host}:{parts.port}/'
        if url != public:
            raise ValueError
        return public
    except (ValueError, TypeError):
        raise ValueError('public loopback origin unavailable') from None


def link_like(line):
    """Conservative even for malformed URLs, embedded URLs and incomplete EOF."""
    # Invalid escapes/control bytes must not turn a recognizable link into
    # persistable output. URL parsing happens only after this conservative test.
    lower = unquote_to_bytes(line).lower().translate(None, bytes(range(33)))
    return (b'/jig/v1/sign-in' in lower or b'secret=' in lower
            or b'secret%3d' in lower
            or (b'http' in lower and (b'sign-in' in lower or b'secret' in lower)))


class Output:
    def __init__(self):
        self.pending = b''

    def feed(self, data, *, eof=False):
        self.pending += data
        lines = self.pending.split(b'\n')
        self.pending = lines.pop()
        framed = [(line + b'\n', True) for line in lines]
        if eof and self.pending:
            framed.append((self.pending, False))
            self.pending = b''
        # The second member is ephemeral: callers may consume a valid engine
        # link, but must persist only the first member and never interpolate it.
        return [(b'' if link_like(line) else line, line if link_like(line) else None, complete)
                for line, complete in framed]


def credential(line):
    """Accept exactly the producer's complete link, all errors are nonsecret."""
    try:
        text = line.decode('ascii')
        if not text.endswith('\n') or '\r' in text or any(c.isspace() for c in text[:-1]):
            raise ValueError
        parts = urlsplit(text[:-1])
        if (parts.path != '/jig/v1/sign-in/link' or parts.fragment
                or re.fullmatch(r'secret=[A-Za-z0-9_-]{43}', parts.query) is None):
            raise ValueError
        secret = parts.query[7:].encode('ascii')
        decoded = base64.urlsafe_b64decode(secret + b'=')
        if len(decoded) != 32 or base64.urlsafe_b64encode(decoded).rstrip(b'=') != secret:
            raise ValueError
        public = origin('http://' + parts.netloc + '/')
        if text[:-1] != public[:-1] + parts.path + '?' + parts.query:
            raise ValueError
        return line, public
    except (ValueError, UnicodeError):
        raise ValueError('engine emitted a malformed sign-in line') from None
