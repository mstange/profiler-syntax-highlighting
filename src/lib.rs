use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    pub type SyntaxParsedFileConstructorOptions;

    #[wasm_bindgen(catch, method, getter)]
    fn checkpointLineInterval(this: &SyntaxParsedFileConstructorOptions) -> Result<usize, JsValue>;

    #[wasm_bindgen(catch, method, getter)]
    fn lineLengthLimit(this: &SyntaxParsedFileConstructorOptions) -> Result<usize, JsValue>;
}

/// Used for displaying syntax-highlighted code in a virtual list.
/// Each line of code needs is converted to a self-contained DOM subtree
/// which marks up various pieces of code with <span> tags and class names.
/// Formatting needs to be supplied by the user; the HTML code does not contain
/// any inline styles.
/// Parsing state is correctly carried across multiple lines. For example, if
/// you have a multi-line /**/ comment from line 1 to line 3, the HTML returned
/// for line 2 will mark up a comment.
/// Performance is best if lines are requested in the right order. This avoids
/// backtracking and repeated parsing.
/// Parsing happens on demand, up to the requested line.
#[wasm_bindgen]
pub struct SyntaxParsedFile(profiler_syntax_highlighting_lib::SyntaxParsedFile);

#[wasm_bindgen]
impl SyntaxParsedFile {
    /// Create a new SyntaxParsedFile.
    #[wasm_bindgen(constructor)]
    #[allow(non_snake_case)]
    pub fn new(
        fileExtension: &str,
        source: String,
        options: SyntaxParsedFileConstructorOptions,
    ) -> Self {
        let mut opts = profiler_syntax_highlighting_lib::Options::default();
        if let Ok(checkpoint_line_interval) = options.checkpointLineInterval() {
            opts.checkpoint_line_interval = checkpoint_line_interval;
        }
        if let Ok(line_length_limit) = options.lineLengthLimit() {
            opts.line_length_limit = line_length_limit;
        }
        Self(profiler_syntax_highlighting_lib::SyntaxParsedFile::new(
            fileExtension, source, opts,
        ))
    }

    /// Returns raw HTML code for the requested line. The code is the source text
    /// of this line interspersed with <span> tags to set class names.
    /// The DOM subtree is self-contained.
    /// `lineIndex` is zero-based.
    #[allow(non_snake_case)]
    pub fn htmlForLine(&mut self, lineIndex: usize) -> Option<String> {
        self.0.html_for_line(lineIndex)
    }
}
