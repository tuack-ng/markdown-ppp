use crate::ast::*;
use crate::typst_printer::ToDoc;
use pretty::{Arena, DocAllocator, DocBuilder};

impl<'a> ToDoc<'a> for Table {
    fn to_doc(&self, state: &'a crate::typst_printer::State<'a>) -> DocBuilder<'a, Arena<'a>, ()> {
        let mut content = state.arena.nil();

        // Add table columns specification
        let column_spec = self
            .alignments
            .iter()
            .map(|align| match align {
                Alignment::Left | Alignment::None => "auto",
                Alignment::Center => "1fr",
                Alignment::Right => "auto",
            })
            .collect::<Vec<_>>()
            .join(", ");
        content = content
            .append(state.arena.text(format!("#table(\n  columns: ({}),", column_spec)));

        // Add header row
        if let Some(header_row) = self.rows.first() {
            content = content.append(state.arena.hardline());
            for cell in header_row.iter() {
                content = content
                    .append(state.arena.text("  ["))
                    .append(cell.to_doc(state).nest(2))
                    .append(state.arena.text("],"));
            }
        }
        content = content.append(state.arena.hardline());

        // Add body rows
        for row in self.rows.iter().skip(1) {
            for cell in row.iter() {
                content = content
                    .append(state.arena.text("  ["))
                    .append(cell.to_doc(state).nest(2))
                    .append(state.arena.text("],"));
            }
            content = content.append(state.arena.hardline());
        }

        content.append(state.arena.text(")"))
    }
}