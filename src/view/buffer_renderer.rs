use rustbox;
use rustbox::{Color, Event, Style};
use scribe::buffer::{Buffer, Lexeme, Position, Range, Token};
use view::color;
use view::terminal::Terminal;

const LINE_LENGTH_GUIDE_OFFSET: usize = 80;
const LINE_WRAPPING: bool = true;
const TAB_WIDTH: usize = 4;

/// A one-time-use type that encapsulates all of the
/// idiosyncracies involved in rendering a buffer to the screen.
pub struct BufferRenderer<'a> {
    alt_background_color: Color,
    buffer: &'a Buffer,
    buffer_position: Position,
    cursor_visible: bool,
    gutter_width: usize,
    highlight: Option<&'a Range>,
    line_number_width: usize,
    screen_position: Position,
    scroll_offset: usize,
    terminal: &'a Terminal,
}

impl<'a> BufferRenderer<'a> {
    pub fn new(buffer: &'a Buffer, scroll_offset: usize, terminal: &'a Terminal, alt_background_color: Color, highlight: Option<&'a Range>) -> BufferRenderer<'a> {
        // Determine the gutter size based on the number of lines.
        let line_number_width = buffer.line_count().to_string().len() + 1;

        BufferRenderer{
            alt_background_color: alt_background_color,
            buffer: buffer,
            cursor_visible: false,
            gutter_width: line_number_width + 2,
            highlight: highlight,
            line_number_width: line_number_width,
            buffer_position: Position{ line: 0, offset: 0 },
            screen_position: Position{ line: 0, offset: 0 },
            scroll_offset: scroll_offset,
            terminal: terminal,
        }
    }

    fn update_positions(&mut self, token: &Token) {
        match token {
            &Token::Newline => self.advance_to_next_line(),
            &Token::Lexeme(ref lexeme) => {
                self.buffer_position = lexeme.position;
                self.screen_position = lexeme.position;
                self.screen_position.offset += self.gutter_width;
            }
        }
    }

    fn on_cursor_line(&self) -> bool {
        self.buffer_position.line == self.buffer.cursor.line
    }

    fn print_line_highlight(&mut self) {
        if self.on_cursor_line() {
            for offset in self.screen_position.offset..self.terminal.width() {
                self.terminal.print_char(offset,
                                self.screen_position.line,
                                rustbox::RB_NORMAL,
                                Color::Default,
                                self.alt_background_color,
                                ' ');
            }
        }
    }

    fn print_length_guide(&mut self) {
        if !self.on_cursor_line() && self.screen_position.offset <= self.length_guide_offset() {
            self.terminal.print_char(self.length_guide_offset(),
                            self.screen_position.line,
                            rustbox::RB_NORMAL,
                            Color::Default,
                            self.alt_background_color,
                            ' ');
        }
    }

    fn length_guide_offset(&self) -> usize {
        self.gutter_width + LINE_LENGTH_GUIDE_OFFSET
    }

    fn advance_to_next_line(&mut self) {
        self.print_line_highlight();
        self.print_length_guide();

        self.buffer_position.line += 1;
        self.buffer_position.offset = 0;
        self.screen_position.line += 1;

        // Draw leading line number for the new line.
        self.screen_position.offset = self.draw_line_number(self.screen_position.line, self.buffer_position.line + 1, self.buffer_position.line == self.buffer.cursor.line, self.line_number_width);
    }

    // Check if we've arrived at the buffer's cursor position,
    // at which point we can set it relative to the screen,
    // which will compensate for scrolling, tab expansion, etc.
    fn set_cursor(&mut self) {
        if *self.buffer.cursor == self.buffer_position {
            self.cursor_visible = true;
            self.terminal.set_cursor(Some(self.screen_position));
        }
    }

    fn current_char_style(&self, token_color: Color) -> (Style, Color) {
        match self.highlight {
            Some(ref highlight_range) => {
                if highlight_range.includes(&self.buffer_position) {
                    (rustbox::RB_REVERSE, Color::Default)
                } else {
                    (rustbox::RB_NORMAL, token_color)
                }
            }
            None => (rustbox::RB_NORMAL, token_color)
        }
    }

    fn background_color(&self) -> Color {
        if self.on_cursor_line() {
            self.alt_background_color
        } else {
            Color::Default
        }
    }

    pub fn print_lexeme(&mut self, lexeme: Lexeme) {
        let token_color = if let Some(ref scope) = lexeme.scope {
            color::map(scope)
        } else {
            Color::Default
        };

        for character in lexeme.value.chars() {
            // We should never run into newline
            // characters, but if we do, ignore them.
            if character == '\n' { continue; }

            self.set_cursor();

            let (style, color) = self.current_char_style(token_color);

            if LINE_WRAPPING && self.screen_position.offset == self.terminal.width() {
                self.screen_position.line += 1;
                self.screen_position.offset = self.gutter_width;
                self.terminal.print_char(self.screen_position.offset, self.screen_position.line, style, color, self.background_color(), character);
                self.screen_position.offset += 1;
                self.buffer_position.offset += 1;
            } else if character == '\t' {
                // Calculate the next tab stop using the tab-aware offset,
                // *without considering the line number gutter*, and then
                // re-add the gutter width to get the actual/screen offset.
                let buffer_tab_stop = next_tab_stop(self.screen_position.offset - self.gutter_width);
                let screen_tab_stop = buffer_tab_stop + self.gutter_width;

                // Print the sequence of spaces and move the offset accordingly.
                for _ in self.screen_position.offset..screen_tab_stop {
                    self.terminal.print_char(self.screen_position.offset, self.screen_position.line, style, color, self.alt_background_color, ' ');
                    self.screen_position.offset += 1;
                }
                self.buffer_position.offset += 1;
            } else {
                self.terminal.print_char(self.screen_position.offset, self.screen_position.line, style, color, self.background_color(), character);
                self.screen_position.offset += 1;
                self.buffer_position.offset += 1;
            }

            self.set_cursor();
        }
    }

    fn before_visible_content(&self) -> bool {
        self.buffer_position.line < self.scroll_offset
    }

    fn after_visible_content(&self) -> bool {
        self.screen_position.line >= self.terminal.height() - 1
    }

    pub fn render(&mut self) {
        // Draw the first line number.
        // Others will be drawn following newline characters.
        self.screen_position.offset = self.draw_line_number(0, self.scroll_offset + 1, self.buffer.cursor.line == self.scroll_offset, self.line_number_width);

        if let Some(tokens) = self.buffer.tokens() {
            'print: for token in tokens.iter() {
                self.update_positions(&token);
                self.set_cursor();

                // Move along until we've hit visible content.
                if self.before_visible_content() {
                    continue;
                }

                // Stop the machine after we've printed all visible content.
                if self.after_visible_content() {
                    break 'print;
                }

                // We're in a visible area.
                if let Token::Lexeme(lexeme) = token {
                    self.print_lexeme(lexeme);
                }
            }

            self.set_cursor();
        }

        // If the cursor was never rendered along with the buffer, we
        // should clear it to prevent its previous value from persisting.
        if !self.cursor_visible {
            self.terminal.set_cursor(None);
        }

        // One last call to these for the last line.
        self.print_line_highlight();
        self.print_length_guide();
    }

    fn draw_line_number(&self, line: usize, line_number: usize, cursor_line: bool, width: usize) -> usize {
        let mut offset = 0;

        // Get left-padded string-based line number.
        let formatted_line_number = format!("{:>width$}  ", line_number, width = width);

        // Print numbers.
        for number in formatted_line_number.chars() {
            // Numbers (and their leading spaces) have background
            // color, but the right-hand side gutter gap does not.
            let background_color = if offset > width && !cursor_line {
                Color::Default
            } else {
                self.alt_background_color
            };

            // Cursor line number is emboldened.
            let weight = if cursor_line {
                rustbox::RB_BOLD
            } else {
                rustbox::RB_NORMAL
            };

            self.terminal.print_char(offset,
                            line,
                            weight,
                            Color::Default,
                            background_color,
                            number);

            offset += 1;
        }
        offset
    }
}

fn next_tab_stop(offset: usize) -> usize {
    (offset / TAB_WIDTH + 1) * TAB_WIDTH
}