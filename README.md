# profiler-syntax-highlighting

This npm module provides syntax highlighting for source code.
It is designed for use in the [Firefox Profiler](https://profiler.firefox.com/).

```js
const source = `#include <iostream>
/**
 * Comment
 */`;
const file = new SyntaxHighlightedFile("cpp", source);
console.log(file.getHtmlForLine(2));
file.free(); // needed to free up wasm memory
```

Output:

```
// TBD
```

### API

[Comming soon]

## 🚴 Usage

### 🛠️ Build with `wasm-pack build`

```
wasm-pack build
```

### 🎁 Publish to NPM with `wasm-pack publish`

```
wasm-pack publish
```
