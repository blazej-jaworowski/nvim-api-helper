use crate::{
    Result, Error,
    nvim::{
        self,
        api::{Buffer, Window},
    },
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
}


pub trait BufferUtils {

    fn max_pos(&self) -> Result<(usize, usize)>;
    fn max_row(&self) -> Result<usize>;
    fn max_row_pos(&self, row: usize) -> Result<(usize, usize)>;
    fn get_window(&self) -> Option<Window>;
    fn get_line(&self, row: usize) -> Result<String>;
    fn get_content(&self) -> Result<String>;
    fn get_cursor(&self) -> Result<(usize, usize)>;
    fn set_cursor(&self, row: usize, col: usize) -> Result<()>;
    fn append_at_position(&mut self, row: usize, col: usize,  text: &str) -> Result<()>;
    fn prepend_at_position(&mut self, row: usize, col: usize, text: &str) -> Result<()>;
    fn append(&mut self, text: &str) -> Result<()>;
    fn prepend(&mut self, text: &str) -> Result<()>;
    fn append_at_cursor(&mut self, text: &str) -> Result<()>;
    fn prepend_at_cursor(&mut self, text: &str) -> Result<()>;

}

impl BufferUtils for Buffer {

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
            return Err(Error::from(BufferError::RowOutOfBounds(row, max_row)));
        }
        Ok(
            self.get_lines(row..(row + 1), true)?.last()
                .expect("Expected line").to_string()
        )
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
        let content = self.get_lines(0..self.line_count()?, true)?
            .join("\n");

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
            return Err(Error::from(BufferError::ColOutOfBounds(col, max_col)));
        }

        col = col.saturating_sub(1);

        Ok(window.set_cursor(row + 1, col)?)
    }

    fn append_at_position(&mut self, row: usize, col: usize,  text: &str) -> Result<()> {
        let max_col = self.max_row_pos(row)?.1;

        if col > max_col {
            return Err(Error::from(BufferError::ColOutOfBounds(col, max_col)));
        }

        self.set_text(row..row, col, col, text.split("\n"))?;

        Ok(())
    }

    fn prepend_at_position(&mut self, row: usize, mut col: usize, text: &str) -> Result<()> {
        let max_col = self.max_row_pos(row)?.1;

        if col > max_col + 1 {
            return Err(Error::from(BufferError::ColOutOfBounds(col, max_col)));
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

}
