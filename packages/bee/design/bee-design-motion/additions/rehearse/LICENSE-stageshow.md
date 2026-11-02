# Stageshow license

The scripts in `rehearse/` are a port of the Stageshow scripts, MIT licensed
by their author. Upstream: https://github.com/benjitaylor/skills, folder
`stageshow/scripts`.

```
MIT License

Copyright (c) 2026 Benji Taylor

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

What changed in the port: the code comments were removed (bee keeps no
comments in code), the stage selector comes from `--select` and defaults to
`body` instead of the Stageshow kit's `.ss-stage` / `.ss-fit` classes, the
Stageshow kit class names survive only as skipped nodes, `sheet.mjs` is new,
and `film.mjs`, `film-checks.mjs`, `motion-watch.mjs` were not ported.
