//! Every name chosen once, after the graph is final, and only from where a type is in the
//! description: a component's key, or the path to its schema. A name never depends on the
//! order types were found in, so one change to a description renames nothing else.
//!
//! A type is declared under a name when the description names it, when stating it needs a name
//! (a recursion; a shape nested past [`MAX_INLINE`] levels), or when several types use it.

mod ident;
mod reach;

use crate::diag::{Diagnostics, Loc};
use crate::id::{DocId, TypeId};
use crate::lower::Lowered;
use crate::program::{Decl, IdentifierRules, Program};
use crate::source::{Documents, node_pointer};

/// How deep shapes nest inline before the innermost is declared under a name of its own, which
/// also bounds how deep a target's printer recurses (G1).
pub const MAX_INLINE: usize = 48;

/// Chooses the names.
#[must_use]
pub fn assign<'t>(
    lowered: Lowered<'t>,
    docs: &Documents<'_>,
    rules: IdentifierRules,
) -> (Program<'t>, Diagnostics) {
    let capabilities = lowered.capabilities();
    let graph = lowered.into_graph();
    let wanted = reach::declared(&graph);
    let mut diags = Diagnostics::default();
    let bases: Vec<String> = wanted
        .iter()
        .map(|w| {
            w.name
                .clone()
                .unwrap_or_else(|| derived(&graph, docs, w.ty))
        })
        .collect();
    let chosen = ident::choose(&bases, rules);
    let mut decls = Vec::with_capacity(wanted.len());
    let mut names = vec![None; graph.len()];
    for (i, (w, c)) in wanted.iter().zip(chosen).enumerate() {
        if c.numbered {
            let at = graph.get(w.ty).origin.map_or(
                Loc {
                    doc: DocId::FIRST,
                    offset: 0,
                },
                |o| docs.loc(o.doc, o.node),
            );
            diags.warn(
                "names.collision",
                at,
                format!(
                    "`{}` names another type too, once made an identifier: declared as `{}`",
                    bases.get(i).map_or("", String::as_str),
                    c.name
                ),
            );
        }
        if let Some(slot) = names.get_mut(w.ty.index()) {
            *slot = Some(i);
        }
        decls.push(Decl {
            name: c.name,
            ty: w.ty,
            named_by_description: w.name.is_some(),
        });
    }
    (
        Program {
            graph,
            decls,
            names,
            capabilities,
        },
        diags,
    )
}

/// A name from where a type's schema is: the path to it without the keywords on the way
/// (`/components/schemas/Pet/properties/owner` is `Pet owner`).
fn derived(graph: &crate::graph::Graph<'_>, docs: &Documents<'_>, ty: TypeId) -> String {
    let Some(loc) = graph.get(ty).origin else {
        return String::from("Merged");
    };
    let Some(doc) = docs.get(loc.doc) else {
        return String::from("Type");
    };
    let pointer = node_pointer(&doc.tree, loc.node);
    let words: Vec<String> = pointer
        .split('/')
        .skip(1)
        .map(|t| t.replace("~1", "/").replace("~0", "~"))
        .filter(|t| !is_structural(t))
        .collect();
    if words.is_empty() {
        String::from("Root")
    } else {
        words.join(" ")
    }
}

fn is_structural(token: &str) -> bool {
    matches!(
        token,
        "components"
            | "schemas"
            | "properties"
            | "items"
            | "prefixItems"
            | "additionalProperties"
            | "patternProperties"
            | "allOf"
            | "anyOf"
            | "oneOf"
            | "not"
            | "$defs"
            | "definitions"
            | "content"
            | "schema"
            | "paths"
            | "then"
            | "else"
            | "if"
            | "additionalItems"
            | "dependentSchemas"
            | "contains"
            | "propertyNames"
            | "unevaluatedItems"
            | "unevaluatedProperties"
            | "contentSchema"
            | "parameters"
            | "headers"
            | "requestBodies"
    ) || token.chars().all(|c| c.is_ascii_digit())
}
