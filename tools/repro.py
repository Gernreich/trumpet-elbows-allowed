#!/usr/bin/env python3
"""Do the Boxes-drawn cut files on disk match what the code draws today?

regress.py runs check.py over the sheets in cut-files/. It measures what is
committed; it never redraws it. So every invariant can hold while the shipped
SVG is one the current code would no longer produce -- which is the same hole
all-gates.sh closed for the ribbon sheets on 2026-09-09 and left open on the
84 files Boxes.py draws, more than twice as many.

Two kinds of folder, checked in opposite directions:

  LIVE      redraw it and the bytes must match. Ordinary reproduction.
  AS-BUILT  the file describes wood that exists. It was deliberately NOT
            regenerated when the sheet was calipered at 2.94mm and the kerf at
            0.13mm, because a record of what was cut is worth more than a file
            nobody is cutting -- see fold2-long-straight/CLAUDE.md. Redrawing
            it therefore SHOULD differ, and what must not change is the file
            itself. Pinned by hash in as-built.sha256.

The as-built direction is the point. Its folders are the two that no ordinary
gate protects: a regenerate sweep would overwrite them, every invariant would
still pass, and the only trace of the instrument that was actually cut would be
gone. `--update` re-pins them, and is the explicit act the prose asks for when
it says to know that you are overwriting the record of the instrument.

The output path is part of every filename -- folder_stack() borrows the family
directory into the slug -- so a design cannot be redrawn into a scratch folder
and compared. Each one is redrawn into a MIRROR of its own path under a temp
root. Redrawing into $TMP directly names every sheet after the temp directory.
"""
import filecmp, hashlib, os, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from regress import DESIGNS, _norm, walk_of

PINS = os.path.join(HERE, 'as-built.sha256')
# Folders whose cut files describe wood that has already been cut. Keep in step
# with the paragraph in fold2-long-straight/CLAUDE.md that names them.
AS_BUILT = {
    'parts/bore/built/coil/fold2-long-straight-3t',
    'parts/bore/concept/walk/no-elbows/coil/no-contact/'
    'fold2-long-straight/coil-10x10x30-1.5t',
}
PY_ = os.environ.get('SNAKEBOX_PY',
                     os.path.expanduser('~/Software/boxes/venv/bin/python'))


def rel_of(folder):
    """The folder as it is written in as-built.sha256: repo-relative, no '..'."""
    return os.path.normpath(os.path.join('tools', folder)).replace('\\', '/')


def sha(path):
    with open(path, 'rb') as fh:
        return hashlib.sha256(fh.read()).hexdigest()


def read_pins():
    pins = {}
    if os.path.exists(PINS):
        for line in open(PINS):
            line = line.strip()
            if line and not line.startswith('#'):
                h, _, name = line.partition('  ')
                pins[name] = h
    return pins


_said = set()


def draw(name, walk, dest, switches):
    """Redraw one design into dest. Returns its cut-files/ or None."""
    os.makedirs(dest, exist_ok=True)
    text = walk_of(walk, HERE)
    r = subprocess.run([PY_, 'bore_split.py', '--write', dest]
                       + text.split() + switches,
                       cwd=HERE, capture_output=True, text=True)
    out = os.path.join(dest, 'cut-files')
    if r.returncode != 0 or not os.path.isdir(out):
        last = ((r.stdout + r.stderr).strip().splitlines() or ['no output'])[-1]
        print(f'  ERROR     {name}: {last[:90]}')
        return None
    # WHAT THE WRITER SAID ON THE WAY PAST. stderr was captured and then thrown
    # away on every successful run, and the generator writes two things there
    # that a person redrawing 84 sheets needs: the warning that the INSTALLED
    # Boxes generator no longer matches tools/ -- so the sheets came out of code
    # that is not in this repository -- and the note that a bore's joint
    # clearance is a guess rather than a measurement. Both were invisible here,
    # which is the one place they were certain to be seen if they were shown at
    # all. Once per distinct message, not once per design, or the drift warning
    # would print 24 times.
    for line in r.stderr.strip().splitlines():
        line = line.strip()
        if line and line not in _said:
            _said.add(line)
            print(f'  note      {line[:100]}')
    return out


def main(update=False):
    pins, bad, same, frozen = read_pins(), 0, 0, 0
    lines = []
    # Every pin this run actually accounted for. The as-built loop walks the
    # files on DISK and asks whether each one is pinned; nothing walked the
    # pins and asked whether each one still has a file. So deleting a sheet
    # that records wood which exists removed it from the comparison and failed
    # nothing -- the one direction this file was written to protect, missed in
    # the one direction nobody looked. Same shape as the deletion a7e36bc that
    # all-gates.sh had to grow a guard for.
    honoured = set()
    with tempfile.TemporaryDirectory() as T:
        for entry in DESIGNS:
            name, walk, folder, switches = _norm(entry)
            if not folder:
                continue
            rel = rel_of(folder)
            sheets = os.path.join(HERE, folder, 'cut-files')
            if not os.path.isdir(sheets):
                print(f'  FAIL      {name}: no cut-files/ under {folder}')
                bad += 1
                continue
            committed = sorted(f for f in os.listdir(sheets)
                               if f.endswith('.svg'))

            if rel in AS_BUILT:
                # Checked by hash, not by redrawing: redrawing SHOULD differ.
                for f in committed:
                    key = f'{rel}/cut-files/{f}'
                    got = sha(os.path.join(sheets, f))
                    honoured.add(key)
                    if update:
                        lines.append(f'{got}  {key}')
                        frozen += 1
                    elif key not in pins:
                        print(f'  FAIL      {name}: {f[:60]} is not pinned')
                        bad += 1
                    elif pins[key] != got:
                        print(f'  CHANGED   {name}: {f[:60]}')
                        print('            this file describes wood that was '
                              'cut. If that was deliberate, re-pin with '
                              'repro.py --update.')
                        bad += 1
                    else:
                        frozen += 1
                if not update:
                    print(f'  frozen    {name:<22} {len(committed)} as cut')
                continue

            out = draw(name, walk, os.path.join(T, rel), switches)
            if out is None:
                bad += 1
                continue
            drawn = sorted(f for f in os.listdir(out) if f.endswith('.svg'))
            for f in sorted(set(committed) ^ set(drawn)):
                where = 'only committed' if f in committed else 'only drawn'
                print(f'  NAME      {name}: {where} {f[:56]}')
                bad += 1
            n = 0
            for f in sorted(set(committed) & set(drawn)):
                if filecmp.cmp(os.path.join(sheets, f),
                               os.path.join(out, f), shallow=False):
                    same += 1
                    n += 1
                else:
                    print(f'  DIFFERS   {name}: {f[:60]}')
                    bad += 1
            print(f'  ok        {name:<22} {n}/{len(committed)}')

    # ... and now the other direction: a pin whose file has gone.
    if not update:
        for key in sorted(set(pins) - honoured):
            print(f'  GONE      {key[:70]}')
            print('            pinned as cut, and no such file is there now. '
                  'If the sheet was deleted on purpose, re-pin with '
                  'repro.py --update.')
            bad += 1

    if update:
        with open(PINS, 'w') as fh:
            fh.write('# The cut files that describe wood which has already '
                     'been cut.\n'
                     '#\n'
                     '# These were NOT regenerated on 2026-09-09 with the rest '
                     'of the tree, when\n'
                     '# the sheet was calipered at 2.94mm and the kerf at '
                     '0.13mm. They are drawn\n'
                     '# at the old 3.0mm and 0.1mm because they record an '
                     'instrument that exists.\n'
                     '# repro.py checks them against these hashes instead of '
                     'redrawing them, so a\n'
                     '# regenerate sweep that overwrites the record fails the '
                     'gate rather than\n'
                     '# passing every invariant in silence.\n'
                     '#\n'
                     '# Rewrite with: repro.py --update -- and only when you '
                     'have decided to\n'
                     '# overwrite the record, which is what '
                     'fold2-long-straight/CLAUDE.md asks.\n')
            for line in sorted(lines):
                fh.write(line + '\n')
        print(f'\n  pinned {frozen} as-built files')
        return 0

    print(f'\n  {same} reproduce, {frozen} frozen as cut, {bad} failing'
          if bad else
          f'\n  {same} reproduce, {frozen} frozen as cut')
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main('--update' in sys.argv[1:]))
