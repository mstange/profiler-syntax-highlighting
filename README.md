# profiler-syntax-highlighting

This npm module provides syntax highlighting for source code.
It is designed for use in the [Firefox Profiler](https://profiler.firefox.com/).

```js
const { SyntaxParsedFile } = await import('profiler-syntax-highlighting');
const source = `#include <iostream>
/**
 * Comment
 */`;
const file = new SyntaxParsedFile("cpp", source);
console.log(file.getHTMLForLine(2));
file.free(); // needed to free up wasm memory
```

Output:

```
<span class="source c++"><span class="comment block c"> * Comment</span></span>
```

## Description

`getHTMLForLine` returns self-contained HTML code for each line of source code.
This makes it perfect for use in a virtualized list.

The HTML code does not contain any inline styles. You need to provide your own
CSS to achieve actual syntax highlighting.

You can generate stylesheets from Sublime Text themes using the `synhtml-css-classes`
example in the [`syntect` repository](https://github.com/trishume/syntect/).

### Motivation

I made this library because none of the other JS syntax highlighting packages
on npm did what I need. The one that came closest to serving my needs was
[`refractor`](https://www.npmjs.com/package/refractor). First I was trying to use it
via [`react-syntax-highlighter`](https://github.com/react-syntax-highlighter/react-syntax-highlighter)
but I think I came to the conclusion that the only way to get virtualized list
rendering with it was to do per-line parsing. So lines in the middle of multi-line
comments would not be known to be inside a comment, because each line was parsed
individually.

This module does exactly what I need, should be quite fast, and should allow
excellent styling abilities because of the versatile "scopes" / class names which
are applied to the code fragments.

### Implementation and Performance

The implementation uses [`syntect`](https://github.com/trishume/syntect/) with the `fancy_regex` backend.
Parsing is done on-demand inside `getHTMLForLine`, synchronously.
For very long files, if your first call to `getHTMLForLine` is for a line number
towards the end of the file, the call might take up to a second to complete.

Repeated parsing is minimized as much as possible by storing checkpoints
at a regular line interval.

## Development

### 🛠️ Build with `wasm-pack build`

```
wasm-pack build
```

### 🎁 Publish to NPM with `wasm-pack publish`

```
wasm-pack publish
```
