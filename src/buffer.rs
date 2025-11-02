use crate::nvim::{
    self,
    api::{Buffer, Window},
};

use itertools::Itertools;

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum BufferError {
    #[error("Buffer not visible")]
    NotVisible,

    #[error("Row out of bounds: {0} ({1})")]
    RowOutOfBounds(usize, usize),

    #[error("Col out of bounds: {0} ({1})")]
    ColOutOfBounds(usize, usize),

    #[error("Nvim api error")]
    NvimError(#[from] nvim::api::Error),
}

type Result<T> = std::result::Result<T, BufferError>;

pub trait BufferUtils {
    fn current_buffer() -> Self;

    fn max_pos(&self) -> Result<(usize, usize)>;
    fn max_row(&self) -> Result<usize>;
    fn max_row_pos(&self, row: usize) -> Result<(usize, usize)>;
    fn get_window(&self) -> Option<Window>;
    fn get_line(&self, row: usize) -> Result<String>;
    fn get_content(&self) -> Result<String>;
    fn get_cursor(&self) -> Result<(usize, usize)>;
    fn set_cursor(&self, row: usize, col: usize) -> Result<()>;
    fn move_cursor(&self, rows: usize, cols: usize) -> Result<()>;
    fn append_at_position(&mut self, row: usize, col: usize, text: &str) -> Result<()>;
    fn prepend_at_position(&mut self, row: usize, col: usize, text: &str) -> Result<()>;
    fn append(&mut self, text: &str) -> Result<()>;
    fn prepend(&mut self, text: &str) -> Result<()>;
    fn append_at_cursor(&mut self, text: &str) -> Result<()>;
    fn prepend_at_cursor(&mut self, text: &str) -> Result<()>;
    fn type_text(&mut self, text: &str) -> Result<()>;
}

impl BufferUtils for Buffer {
    fn current_buffer() -> Buffer {
        Buffer::current()
    }

    fn max_pos(&self) -> Result<(usize, usize)> {
        self.max_row_pos(self.max_row()?)
    }

    fn max_row(&self) -> Result<usize> {
        Ok(self.line_count()? - 1)
    }

    fn max_row_pos(&self, row: usize) -> Result<(usize, usize)> {
        let line = self.get_line(row)?;
        let line_len = line.len();

        if line_len == 0 {
            Ok((row, 0))
        } else {
            Ok((row, line_len))
        }
    }

    fn get_line(&self, row: usize) -> Result<String> {
        let max_row = self.max_row()?;
        if row > max_row {
            return Err(BufferError::RowOutOfBounds(row, max_row));
        }
        Ok(self
            .get_lines(row..(row + 1), true)?
            .last()
            .expect("Expected line")
            .to_string())
    }

    fn get_window(&self) -> Option<Window> {
        nvim::api::list_wins().find(|win| {
            if let Ok(buf) = win.get_buf() {
                buf == *self
            } else {
                false
            }
        })
    }

    fn get_content(&self) -> Result<String> {
        let content = self.get_lines(0..self.line_count()?, true)?.join("\n");

        Ok(content)
    }

    fn get_cursor(&self) -> Result<(usize, usize)> {
        let window = self.get_window().ok_or(BufferError::NotVisible)?;
        let (cursor_row, cursor_col) = window.get_cursor()?;
        let cursor_row = cursor_row - 1;

        if self.get_line(cursor_row)?.is_empty() {
            Ok((cursor_row, 0))
        } else {
            Ok((cursor_row, cursor_col + 1))
        }
    }

    fn set_cursor(&self, row: usize, mut col: usize) -> Result<()> {
        let mut window = self.get_window().ok_or(BufferError::NotVisible)?;

        let max_col = self.max_row_pos(row)?.1;

        if col > max_col {
            return Err(BufferError::ColOutOfBounds(col, max_col));
        }

        col = col.saturating_sub(1);

        Ok(window.set_cursor(row + 1, col)?)
    }

    fn move_cursor(&self, rows: usize, cols: usize) -> Result<()> {
        let (mut row, mut col) = self.get_cursor()?;
        row += rows;
        col += cols;
        self.set_cursor(row, col)
    }

    fn append_at_position(&mut self, row: usize, col: usize, text: &str) -> Result<()> {
        let max_col = self.max_row_pos(row)?.1;

        if col > max_col {
            return Err(BufferError::ColOutOfBounds(col, max_col));
        }

        self.set_text(row..row, col, col, text.split("\n"))?;

        Ok(())
    }

    fn prepend_at_position(&mut self, row: usize, mut col: usize, text: &str) -> Result<()> {
        let max_col = self.max_row_pos(row)?.1;

        if col > max_col + 1 {
            return Err(BufferError::ColOutOfBounds(col, max_col));
        }

        col = col.saturating_sub(1);

        self.set_text(row..row, col, col, text.split("\n"))?;

        Ok(())
    }

    fn append(&mut self, text: &str) -> Result<()> {
        let (max_row, max_col) = self.max_pos()?;
        self.append_at_position(max_row, max_col, text)
    }

    fn prepend(&mut self, text: &str) -> Result<()> {
        self.prepend_at_position(0, 0, text)
    }

    fn append_at_cursor(&mut self, text: &str) -> Result<()> {
        let (cursor_row, cursor_col) = self.get_cursor()?;
        self.append_at_position(cursor_row, cursor_col, text)
    }

    fn prepend_at_cursor(&mut self, text: &str) -> Result<()> {
        let (cursor_row, cursor_col) = self.get_cursor()?;
        self.prepend_at_position(cursor_row, cursor_col, text)
    }

    fn type_text(&mut self, text: &str) -> Result<()> {
        self.append_at_cursor(text)?;

        let lines: Vec<&str> = text.split("\n").collect();

        let move_rows = lines.len() - 1;
        let move_cols = lines.last().expect("text is not empty").len();

        self.move_cursor(move_rows, move_cols)
    }
}

#[cfg(feature = "nvim_tests")]
pub mod nvim_tests {
    use super::*;

    const INIT_CONTENT: &str = r#"First line
Second line
Third line!"#;

    fn prepare_test_content() -> Result<Buffer> {
        let mut buffer = Buffer::current_buffer();
        buffer.append(INIT_CONTENT)?;

        Ok(buffer)
    }

    macro_rules! assert_content {
        ($content:expr) => {{
            let buffer = Buffer::current_buffer();
            let content = buffer.get_content()?;
            assert_eq!(content, $content)
        }};
    }

    macro_rules! assert_buffer_error {
        ($value:expr, $error:expr) => {
            if let Err(e) = $value {
                assert_eq!(e, $error)
            } else {
                assert!(false, "Expected buffer error, got: {:?}", $value)
            }
        };
    }

    #[nvim::test(nvim_oxi = nvim)]
    fn test_buffer_append() -> Result<()> {
        assert_content!("");

        let mut buffer = Buffer::current_buffer();

        buffer.append("First line")?;
        assert_content!("First line");

        buffer.append("\nSecond line")?;
        assert_content!("First line\nSecond line");

        buffer.prepend("Actual first line\n")?;
        assert_content!("Actual first line\nFirst line\nSecond line");

        Ok(())
    }

    #[nvim::test(nvim_oxi = nvim)]
    fn test_buffer_cursor() -> Result<()> {
        let mut buffer = Buffer::current_buffer();

        assert_eq!(buffer.get_cursor()?, (0, 0));

        buffer.append(INIT_CONTENT)?;

        buffer.set_cursor(1, 4)?;
        assert_eq!(buffer.get_cursor()?, (1, 4));

        // Cursor position can be 0 only when the line is empty
        buffer.set_cursor(0, 0)?;
        assert_eq!(buffer.get_cursor()?, (0, 1));

        buffer.set_cursor(2, 11)?;
        assert_eq!(buffer.get_cursor()?, (2, 11));

        assert_buffer_error!(buffer.set_cursor(3, 0), BufferError::RowOutOfBounds(3, 2));
        assert_buffer_error!(
            buffer.set_cursor(1, 12),
            BufferError::ColOutOfBounds(12, 11)
        );

        Ok(())
    }

    #[nvim::test(nvim_oxi = nvim)]
    fn test_buffer_cursor_append() -> Result<()> {
        let mut buffer = prepare_test_content()?;

        buffer.set_cursor(1, 7)?;
        buffer.append_at_cursor("test ")?;

        assert_content!(
            r#"First line
Second test line
Third line!"#
        );

        buffer.set_cursor(2, 7)?;
        buffer.prepend_at_cursor("test ")?;

        assert_content!(
            r#"First line
Second test line
Third test line!"#
        );

        Ok(())
    }

    #[nvim::test(nvim_oxi = nvim)]
    fn test_buffer_pos_append() -> Result<()> {
        let mut buffer = prepare_test_content()?;

        buffer.append_at_position(1, 7, "test ")?;

        assert_content!(
            r#"First line
Second test line
Third line!"#
        );

        buffer.append_at_position(2, 11, " :)")?;

        assert_content!(
            r#"First line
Second test line
Third line! :)"#
        );

        assert_buffer_error!(
            buffer.append_at_position(3, 0, ":("),
            BufferError::RowOutOfBounds(3, 2)
        );
        assert_buffer_error!(
            buffer.append_at_position(1, 17, ":("),
            BufferError::ColOutOfBounds(17, 16)
        );

        buffer.prepend_at_position(1, 17, " ;)")?;

        assert_content!(
            r#"First line
Second test line ;)
Third line! :)"#
        );

        buffer.prepend_at_position(0, 0, "Actual first line\n")?;

        assert_content!(
            r#"Actual first line
First line
Second test line ;)
Third line! :)"#
        );

        assert_buffer_error!(
            buffer.prepend_at_position(4, 0, ":("),
            BufferError::RowOutOfBounds(4, 3)
        );

        Ok(())
    }

    #[nvim::test(nvim_oxi = nvim)]
    fn test_buffer_pos() -> Result<()> {
        let buffer = prepare_test_content()?;

        assert_eq!(buffer.max_row()?, 2);
        assert_eq!(buffer.max_row_pos(0)?, (0, 10));
        assert_eq!(buffer.max_row_pos(2)?, (2, 11));
        assert_eq!(buffer.max_pos()?, (2, 11));

        Ok(())
    }
}
