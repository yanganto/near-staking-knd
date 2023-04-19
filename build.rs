use std::error::Error;
use vergen::EmitBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    EmitBuilder::builder()
        .git_commit_date()
        .git_branch()
        .git_describe(true, true, None)
        .emit()?;
    Ok(())
}
