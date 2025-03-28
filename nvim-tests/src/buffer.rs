use nvim_api_helper::{
    Result,
    nvim::{
        self,
        api::Buffer,
    },
    buffer::{BufferUtils, BufferError},
};

const INIT_CONTENT: &str =
r#"First line
Second line
Third line!"#;

fn prepare_test_content() -> Result<Buffer> {
    let mut buffer = Buffer::current();
    buffer.append(INIT_CONTENT)?;

    Ok(buffer)
}

macro_rules! assert_content {
    ($content:expr) => {{
        let buffer = nvim_api_helper::nvim::api::Buffer::current();
        let content = buffer.get_content()?;
        assert_eq!(content, $content)
    }}
}

macro_rules! assert_buffer_error {
    ($value:expr, $error:expr) => {
        if let Err(nvim_api_helper::Error::Buffer(e)) = $value {
            assert_eq!(e, $error)
        } else {
            assert!(false, "Expected buffer error, got: {:?}", $value)
        }
    }
}


#[nvim::test(nvim_oxi = nvim)]
fn test_buffer_append() -> Result<()> {
    assert_content!("");

    let mut buffer = Buffer::current();

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
    let mut buffer = Buffer::current();

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
    assert_buffer_error!(buffer.set_cursor(1, 12), BufferError::ColOutOfBounds(12, 11));

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

    assert_buffer_error!(buffer.append_at_position(3, 0, ":("), BufferError::RowOutOfBounds(3, 2));
    assert_buffer_error!(buffer.append_at_position(1, 17, ":("), BufferError::ColOutOfBounds(17, 16));

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

    assert_buffer_error!(buffer.prepend_at_position(4, 0, ":("), BufferError::RowOutOfBounds(4, 3));

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
