use std::fmt;
use std::fmt::Write;
use syntect::parsing::{
    BasicScopeStackOp, ParseState, Scope, ScopeStack, ScopeStackOp, SyntaxReference, SyntaxSet,
    SCOPE_REPO,
};
use syntect::util::LinesWithEndings;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref SYNTAX_SET: &'static SyntaxSet =
        Box::leak(Box::new(SyntaxSet::load_defaults_newlines()));
}

/// Wrapper struct which will emit the HTML-escaped version of the contained
/// string when passed to a format string.
struct Escape<'a>(pub &'a str);

impl<'a> fmt::Display for Escape<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Because the internet is always right, turns out there's not that many
        // characters to escape: http://stackoverflow.com/questions/7381974
        let Escape(s) = *self;
        let pile_o_bits = s;
        let mut last = 0;
        for (i, ch) in s.bytes().enumerate() {
            match ch as char {
                '<' | '>' | '&' | '\'' | '"' => {
                    fmt.write_str(&pile_o_bits[last..i])?;
                    let s = match ch as char {
                        '>' => "&gt;",
                        '<' => "&lt;",
                        '&' => "&amp;",
                        '\'' => "&#39;",
                        '"' => "&quot;",
                        _ => unreachable!(),
                    };
                    fmt.write_str(s)?;
                    last = i + 1;
                }
                _ => {}
            }
        }

        if last < s.len() {
            fmt.write_str(&pile_o_bits[last..])?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct SyntaxParsedFile {
    /// The syntax for this file.
    syntax: &'static SyntaxReference,
    /// The entire contents of the file.
    source: String,
    /// Checkpoints taken at a regular interval, up to the last line that we needed to parse so far.
    interval_checkpoints: IntervalCheckpoints,
    /// The checkpoint taken after the last line that we were asked to parse.
    /// Allows efficient continuing of parsing if we get called for sequential lines in the right order.
    last_call_checkpoint: Checkpoint,
    /// The maxmimum length of a line that we still parse. Lines longer than this will be plain-text without highlighting.
    line_length_limit: usize,
}

impl SyntaxParsedFile {
    pub fn new(file_ext: &str, source: String, options: Options) -> Self {
        let syntax = SYNTAX_SET
            .find_syntax_by_extension(file_ext)
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
        let parse_state = ParseState::new(syntax);
        let scope_stack = ScopeStack::new();
        let checkpoint = Checkpoint {
            line_index: 0,
            str_offset: 0,
            parse_state,
            scope_stack,
        };
        SyntaxParsedFile {
            syntax,
            source,
            interval_checkpoints: IntervalCheckpoints::new(
                options.checkpoint_line_interval,
                checkpoint.clone(),
            ),
            last_call_checkpoint: checkpoint,
            line_length_limit: options.line_length_limit,
        }
    }

    pub fn html_for_line(&mut self, line_index: usize) -> Option<String> {
        let mut checkpoint = self.checkpoint_for_line(line_index)?;
        let line = LinesWithEndings::from(&self.source[checkpoint.str_offset..]).next()?;
        if line.trim().is_empty() {
            return Some(String::new());
        }

        if line.len() > self.line_length_limit {
            // For long lines, give up on parsing and just return it with no markup.
            return Some(format!("{}", Escape(line)));
        }

        let mut s = String::new();
        let mut open_span_count = 0;

        // Detect and skip empty inner <span> tags.
        let mut span_empty = false;
        let mut span_start = 0;

        // Open spans for any scopes that are on the stack at the beginning of the line.
        for scope in checkpoint.scope_stack.as_slice() {
            span_start = s.len();
            span_empty = true;
            s.push_str("<span class=\"");
            scope_to_classes(&mut s, scope);
            s.push_str("\">");
            open_span_count += 1;
        }

        // The offset in `line` up to which we have outputed the line contents to `s`.
        let mut cur_index = 0;

        // Parse the line and consume the scope ops.
        checkpoint.step_line(line, |stack, ops| {
            for &(i, ref op) in &ops {
                if i > cur_index {
                    span_empty = false;
                    write!(s, "{}", Escape(&line[cur_index..i])).unwrap();
                    cur_index = i
                }
                stack.apply_with_hook(op, |basic_op, _| match basic_op {
                    BasicScopeStackOp::Push(scope) => {
                        span_start = s.len();
                        span_empty = true;
                        s.push_str("<span class=\"");
                        scope_to_classes(&mut s, &scope);
                        s.push_str("\">");
                        open_span_count += 1;
                    }
                    BasicScopeStackOp::Pop => {
                        if !span_empty {
                            s.push_str("</span>");
                        } else {
                            s.truncate(span_start);
                        }
                        open_span_count -= 1;
                        span_empty = false;
                    }
                });
            }
        });

        // Write out the rest of the string, and close all open spans.
        write!(s, "{}", Escape(line[cur_index..line.len()].trim_end())).unwrap();
        for _ in 0..open_span_count {
            s.push_str("</span>");
        }

        self.interval_checkpoints.maybe_store(&checkpoint);
        self.last_call_checkpoint = checkpoint;

        Some(s)
    }

    fn checkpoint_for_line(&mut self, line_index: usize) -> Option<Checkpoint> {
        if self.last_call_checkpoint.line_index == line_index {
            return Some(self.last_call_checkpoint.clone());
        }

        let interval_checkpoint = self.interval_checkpoints.closest(line_index);
        let starting_checkpoint = if self.last_call_checkpoint.line_index < line_index
            && self.last_call_checkpoint.line_index > interval_checkpoint.line_index
        {
            self.last_call_checkpoint.clone()
        } else {
            interval_checkpoint.clone()
        };

        let checkpoint = self.advance_checkpoint(starting_checkpoint, line_index);
        if checkpoint.line_index == line_index {
            Some(checkpoint)
        } else {
            None
        }
    }

    fn advance_checkpoint(
        &mut self,
        mut checkpoint: Checkpoint,
        target_line_index: usize,
    ) -> Checkpoint {
        if checkpoint.line_index == target_line_index {
            return checkpoint;
        }

        for line in LinesWithEndings::from(&self.source[checkpoint.str_offset..]) {
            if line.len() > self.line_length_limit {
                checkpoint.step_overlong_line(line, self.syntax);
            } else {
                checkpoint.step_line_simple(line);
            }
            self.interval_checkpoints.maybe_store(&checkpoint);

            if checkpoint.line_index == target_line_index {
                break;
            }
        }

        checkpoint
    }
}

#[derive(Clone, Debug)]
struct IntervalCheckpoints {
    /// The interval (in number of lines) at which to capture a checkpoint.
    interval: usize,
    /// The checkpoints. `checkpoints[i]` has `line_index == i * interval`
    checkpoints: Vec<Checkpoint>,
}

impl IntervalCheckpoints {
    /// Create an IntervalCheckpoints instance.
    /// `first` must describe the start of the first line.
    pub fn new(interval: usize, first: Checkpoint) -> Self {
        Self {
            interval,
            checkpoints: vec![first],
        }
    }

    /// Return the closest interval checkpoint at or before line_index.
    pub fn closest(&self, line_index: usize) -> &Checkpoint {
        let closest_index = line_index / self.interval;
        self.checkpoints
            .get(closest_index)
            .unwrap_or_else(|| self.checkpoints.last().unwrap())
    }

    /// Stores this checkpoint if it is at an "interval line" and if it is
    /// the first time we've gotten this far.
    pub fn maybe_store(&mut self, checkpoint: &Checkpoint) {
        if checkpoint.line_index == self.checkpoints.len() * self.interval {
            self.checkpoints.push(checkpoint.clone());
        }
    }
}

/// Create a space-separated string for the scope, to be used in the
/// HTML class attribute.
fn scope_to_classes(s: &mut String, scope: &Scope) {
    let repo = SCOPE_REPO.lock().unwrap();
    for i in 0..(scope.len()) {
        let atom = scope.atom_at(i as usize);
        let atom_s = repo.atom_str(atom);
        if i != 0 {
            s.push(' ')
        }
        s.push_str(atom_s);
    }
}

/// Stores the parse state and scope stack at the start of a line.
/// This allows efficient continuation of parsing from this point.
#[derive(Clone, Debug)]
struct Checkpoint {
    /// The checkpoint describes the state at the start of this line.
    line_index: usize,
    /// The position in the file (in bytes) at which this line starts.
    str_offset: usize,
    /// The parse state at the start of this line.
    parse_state: ParseState,
    /// The scope stack at the start of this line.
    scope_stack: ScopeStack,
}

impl Checkpoint {
    /// Parse this line and update the state.
    pub fn step_line_simple(&mut self, line: &str) {
        self.step_line(line, |stack, ops| {
            for (_pos, op) in &ops {
                stack.apply(op);
            }
        });
    }

    /// Parse this line and delegate updating the scope stack to a callback.
    /// This can be used to efficiently combine the state stack update with
    /// any other processing of the scope ops.
    /// The callback `f` needs to make sure that the scope stack is updated
    /// with the scope ops.
    pub fn step_line<F>(&mut self, line: &str, mut f: F)
    where
        F: FnMut(&mut ScopeStack, Vec<(usize, ScopeStackOp)>),
    {
        let scope_ops = self.parse_state.parse_line(line, &SYNTAX_SET);
        f(&mut self.scope_stack, scope_ops);
        self.str_offset += line.len();
        self.line_index += 1;
    }

    /// For very long lines, give up on parsing and just reset the state to zero.
    pub fn step_overlong_line(&mut self, line: &str, syntax: &SyntaxReference) {
        self.parse_state = ParseState::new(syntax);
        self.scope_stack = ScopeStack::new();
        self.str_offset += line.len();
        self.line_index += 1;
    }
}

#[derive(Clone, Debug)]
pub struct Options {
    /// The interval (in number of lines) at which to capture a checkpoint, to
    /// speed up "seeking" during parsing.
    pub checkpoint_line_interval: usize,
    /// The maxmimum length of a line that we still parse. Lines longer than
    /// this will be plain-text without highlighting.
    pub line_length_limit: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            checkpoint_line_interval: 100,
            line_length_limit: 1000,
        }
    }
}
