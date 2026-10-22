"""Pack a gallery printout as deterministic, uncompressed USTAR bytes."""
import io
from pathlib import Path
import stat
import tarfile


def pack(directory):
    """Include sorted relative names and fixed metadata; refuse special files.

    USTAR is the only format: names it cannot represent are refused rather
    than silently switching formats or adding extended headers.
    """
    directory = Path(directory)
    if not stat.S_ISDIR(directory.lstat().st_mode):
        raise ValueError(f'gallery pack: expected a directory: {directory}')
    buffer = io.BytesIO()
    with tarfile.open(fileobj=buffer, mode='w', format=tarfile.USTAR_FORMAT) as archive:
        for path in sorted(directory.rglob('*'),
                           key=lambda path: path.relative_to(directory).as_posix()):
            metadata = path.lstat()
            name = path.relative_to(directory).as_posix()
            member = tarfile.TarInfo(name)
            member.mtime = member.uid = member.gid = 0
            member.uname = member.gname = ''
            if stat.S_ISDIR(metadata.st_mode):
                member.type = tarfile.DIRTYPE
                member.mode = 0o755
                archive.addfile(member)
            elif stat.S_ISREG(metadata.st_mode):
                member.mode = 0o644
                member.size = metadata.st_size
                with path.open('rb') as contents:
                    archive.addfile(member, contents)
            else:
                raise ValueError(f'gallery pack: not a regular file or directory: {name}')
    return buffer.getvalue()
