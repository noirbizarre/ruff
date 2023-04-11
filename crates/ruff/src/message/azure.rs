use crate::message::{Emitter, EmitterContext, Message};
use crate::registry::AsRule;
use ruff_python_ast::source_code::{OneIndexed, SourceLocation};
use std::io::Write;

/// Generate error logging commands for Azure Pipelines format.
/// See [documentation](https://learn.microsoft.com/en-us/azure/devops/pipelines/scripts/logging-commands?view=azure-devops&tabs=bash#logissue-log-an-error-or-warning)
#[derive(Default)]
pub struct AzureEmitter;

impl Emitter for AzureEmitter {
    fn emit(
        &mut self,
        writer: &mut dyn Write,
        messages: &[Message],
        context: &EmitterContext,
    ) -> anyhow::Result<()> {
        for message in messages {
            let location = if context.is_jupyter_notebook(message.filename()) {
                // We can't give a reasonable location for the structured formats,
                // so we show one that's clearly a fallback
                SourceLocation {
                    row: OneIndexed::from_zero_indexed(0),
                    column: OneIndexed::from_zero_indexed(0),
                }
            } else {
                message.compute_start_location()
            };

            writeln!(
                writer,
                "##vso[task.logissue type=error\
                        ;sourcepath={filename};linenumber={line};columnnumber={col};code={code};]{body}",
                filename = message.filename(),
                line = location.row,
                col = location.row,
                code = message.kind.rule().noqa_code(),
                body = message.kind.body,
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::message::tests::{capture_emitter_output, create_messages};
    use crate::message::AzureEmitter;
    use insta::assert_snapshot;

    #[test]
    fn output() {
        let mut emitter = AzureEmitter::default();
        let content = capture_emitter_output(&mut emitter, &create_messages());

        assert_snapshot!(content);
    }
}
