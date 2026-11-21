# Presenting

Presentation mode shows each page of your document as a full-screen slide. It works with any document, but looks best with a slide-sized page. In **Page settings**, choose a **16:9** or **4:3 presentation** page size.

Start it with the **Present** button in the editor toolbar. Click **Exit** to stop. ++esc++ only leaves full screen; you keep presenting in the tab, and **Full screen** at the top brings it back. The arrow next to **Present** also offers a [speaker view](#speaker-view).

## Moving between slides

- Use the arrow keys, ++page-up++ and ++page-down++, or ++space++ and ++backspace++.
- ++home++ jumps to the first slide and ++end++ to the last.
- The arrows and slide number at the bottom of the screen do the same.
- A thin bar along the bottom shows how far through you are.

## All slides

Press ++g++ or ++esc++, click the slide number, or choose **All slides** at the top to see every slide at once. Click one to jump to it.

## Drawing on slides

The bottom bar has drawing tools. Each slide keeps its own drawings until you leave presentation mode.

| Tool | Key | What it does |
|---|---|---|
| Laser pointer | ++l++ | A red dot that follows your mouse. |
| Pen | ++p++ | Draw thin lines. |
| Highlighter | ++h++ | Draw wide, see-through strokes. |
| Eraser | ++e++ | Rub out drawings. |

Pick a color next to the pen and highlighter. Undo with ++ctrl+z++, redo with ++ctrl+shift+z++, and clear the whole slide with ++c++.

## Timer

The timer at the top left keeps track of how long you have been talking. It starts paused; press play when you begin. Pause or reset it at any time.

## Staying out of the way

The controls fade away after a few seconds without moving the mouse, and come back as soon as you move it.

## Speaker view

Choose **Speaker view** from the arrow next to **Present**, or type `/speaker` in the editor, to give a talk from two screens. The slides open in a new window for the audience, and this tab becomes your speaker view, with the current slide, the next slide, your notes and the timer.

1. Drag the new window to the projector or shared screen, move the mouse over it and click **Full screen** at the top right.
2. Present from the speaker view. Moving between slides, the laser pointer and drawings all show up in the audience window.

If your browser blocks the new window, allow pop-ups for this site and click **Open audience window**.

| Key | What it does |
|---|---|
| ++b++ | Black out the audience screen. You can move to another slide without the audience seeing it; press ++b++ again to show your current slide. |
| ++f++ | Freeze the audience screen on the slide it shows, while you look ahead. Press ++f++ again to catch up. |

Keys pressed in the audience window, for example from a presentation clicker, work the same as in the speaker view. Click **Exit** to end the presentation and close the audience window.

### Speaker notes

The speaker view shows notes written with a slide package:

- [Touying](https://touying-typ.github.io/): `#speaker-note[Mention the survey results.]`
- [Polylux](https://polylux.dev/book/): `#pdfpc.speaker-note("Mention the survey results.")`
- [Mosaic](https://vincentarelbundock.github.io/mosaic/presenting/notes.html): `#m.note[Mention the survey results.]`

Without a slide package, place `#metadata((t: "Note", v: "Mention the survey results.")) <pdfpc>` on the page the note belongs to.

Notes appear only in the speaker view. If you use Touying's `show-notes-on-second-screen`, turn it off: the speaker view already shows the notes, and the setting would also draw them on every slide.
