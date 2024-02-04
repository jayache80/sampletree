# sampletree

Lightweight, text-based user interface (TUI) utility to recurse a directory and
audition sound files.

## Install

    cargo install --path .

## Use

In a shell, run:

    sampletree <path/to/directory>

For example:

    sampletree Drum\ Machines/

A user interface similar to the following should appear:

    Traversing Drum Machines/
    z[ ] x[ ] c[ ] v[ ]
    [1/4433] Drum Machines/Alesis SR-16/SR16 Kik_Amb Elect.wav

Each sample is played as the user traverses the directory. Samples can be
assigned to "pad" keys <kbd>z</kbd>, <kbd>x</kbd>, <kbd>c</kbd>, and
<kbd>v</kbd> by pressing <kbd>Shift</kbd>+<kbd>{z,x,c,y}</kbd>, and played back
by pressing the pad key after a sample has been assigned. The pads may be used
as a primative drum machine to assist in auditioning the samples.

When exiting the program, the paths to the sample files assigned to the pads
are dumped to stdout.

| Key                                          | Function                                  |
|----------------------------------------------|-------------------------------------------|
| <kbd>j</kbd> or <kbd>i</kbd>                 | Play next sample                          |
| <kbd>k</kbd> or <kbd>o</kbd>                 | Play previous sample                      |
| <kbd>Shift></kbd>+<kbd>z</kbd>               | Assign current sample to <kbd>z</kbd> key |
| <kbd>z</kbd>                                 | Play sample assigned to <kbd>z</kbd> key  |
| <kbd>Shift></kbd>+<kbd>x</kbd>               | Assign current sample to <kbd>x</kbd> key |
| <kbd>x</kbd>                                 | Play sample assigned to <kbd>x</kbd> key  |
| <kbd>Shift></kbd>+<kbd>c</kbd>               | Assign current sample to <kbd>c</kbd> key |
| <kbd>c</kbd>                                 | Play sample assigned to <kbd>c</kbd> key  |
| <kbd>Shift></kbd>+<kbd>v</kbd>               | Assign current sample to <kbd>v</kbd> key |
| <kbd>v</kbd>                                 | Play sample assigned to <kbd>v</kbd> key  |
| <kbd>s</kbd>                                 | Stop all sample playback                  |
| <kbd>q</kbd> or <kbd>Ctrl</kbd>+<kbd>c</kbd> | Exit                                      |
| `<any other key>`                            | Play current sample                       |
