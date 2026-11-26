//! The package: `package.json`, the TypeScript configs and the license.
//!
//! One package from the repository's root, built twice by `tsc` alone (no bundler, no tool of
//! someone else's): ES modules into `dist/esm`, CommonJS into `dist/cjs` (with a
//! `package.json` of `"type": "commonjs"` there, which `scripts/build` writes), each with its
//! declarations beside it, and `exports` sending `import` and `require` each to its own build
//! and types. Dual rather than ESM alone, as the study chose (04, section 1): `engines` allows
//! Node 20 before 20.19, which cannot `require` an ES module, and Are The Types Wrong, which the
//! judge runs, counts an ES module behind `require` as a problem. `typescript` is the one
//! development dependency, at the version flowapi's own checks compile clients with; nothing at
//! run time (the client calls the platform's `fetch`).

use super::{Ctx, RepoFile, generated_file};
use crate::config::issue::Issues;
use crate::doc::ROOT;
use crate::out::json_str;

/// The TypeScript the package builds and checks with: `conformance/ts`'s.
pub(crate) const TYPESCRIPT: &str = "7.0.2";

/// The license texts the generator has, by SPDX id; `{package}` is the package's name.
const LICENSES: &[(&str, &str)] = &[
    ("Apache-2.0", include_str!("files/LICENSE-Apache-2.0")),
    ("MIT", include_str!("files/LICENSE-MIT")),
];

/// The compiler options every config shares: what `tests/client.mjs` holds the client to.
const STRICT: &str = r#""target": "es2022",
    "lib": ["es2022"],
    "module": "nodenext",
    "strict": true,
    "exactOptionalPropertyTypes": true,
    "types": []"#;

pub(crate) fn files(ctx: &Ctx<'_>, issues: &mut Issues) -> Vec<RepoFile> {
    let mut out = vec![
        generated_file("package.json", package_json(ctx)),
        generated_file("tsconfig.json", tsconfig()),
        generated_file(
            "tsconfig.build.json",
            String::from(concat!(
                "{\n  \"extends\": \"./tsconfig.json\",\n  \"compilerOptions\": {\n",
                "    \"noEmit\": false,\n    \"declaration\": true,\n",
                "    \"rootDir\": \"src\",\n    \"outDir\": \"dist/esm\"\n  }\n}\n",
            )),
        ),
        generated_file(
            "tsconfig.cjs.json",
            String::from(concat!(
                "{\n  \"extends\": \"./tsconfig.build.json\",\n  \"compilerOptions\": {\n",
                "    \"module\": \"commonjs\",\n    \"outDir\": \"dist/cjs\"\n  }\n}\n",
            )),
        ),
        generated_file(
            "tsconfig.dist.json",
            format!(
                "{{\n  \"compilerOptions\": {{\n    {STRICT},\n    \"skipLibCheck\": false,\n    \"noEmit\": true\n  }},\n  \"include\": [\"dist/**/*.d.ts\"]\n}}\n"
            ),
        ),
    ];
    if let Some(license) = license(ctx, issues) {
        out.push(generated_file("LICENSE", license));
    }
    out
}

fn tsconfig() -> String {
    format!(
        "{{\n  \"compilerOptions\": {{\n    {STRICT},\n    \"rootDir\": \"src\",\n    \"noEmit\": true\n  }},\n  \"include\": [\"src\"]\n}}\n"
    )
}

/// The description's title, for the package's description.
pub(crate) fn title(ctx: &Ctx<'_>) -> Option<String> {
    let doc = &ctx.ops.doc;
    doc.get(ROOT, "info").and_then(|i| doc.str(i, "title"))
}

fn package_json(ctx: &Ctx<'_>) -> String {
    let description = title(ctx).map_or_else(
        || String::from("A TypeScript client of the API"),
        |t| format!("A TypeScript client of the {t} API"),
    );
    let s = &ctx.config.settings;
    let mut fields = vec![
        format!("\"name\": {}", json_str(ctx.package())),
        format!("\"version\": {}", json_str(ctx.version())),
        format!("\"description\": {}", json_str(&description)),
    ];
    if let Some(l) = &ctx.config.license {
        fields.push(format!("\"license\": {}", json_str(&l.name)));
    }
    if let Some(r) = &s.repository {
        fields.push(format!(
            "\"repository\": {{ \"type\": \"git\", \"url\": {} }}",
            json_str(r)
        ));
    }
    fields.extend(
        [
            "\"type\": \"module\"",
            "\"sideEffects\": false",
            "\"engines\": { \"node\": \">=20\" }",
            "\"main\": \"./dist/cjs/index.js\"",
            "\"types\": \"./dist/cjs/index.d.ts\"",
            concat!(
                "\"exports\": {\n    \".\": {\n",
                "      \"import\": { \"types\": \"./dist/esm/index.d.ts\", \"default\": \"./dist/esm/index.js\" },\n",
                "      \"require\": { \"types\": \"./dist/cjs/index.d.ts\", \"default\": \"./dist/cjs/index.js\" }\n",
                "    },\n    \"./package.json\": \"./package.json\"\n  }"
            ),
            "\"files\": [\"dist\"]",
            concat!(
                "\"scripts\": {\n    \"build\": \"scripts/build\",\n    \"lint\": \"scripts/lint\",\n",
                "    \"test\": \"scripts/test\",\n    \"check\": \"scripts/check\"\n  }"
            ),
        ]
        .map(str::to_owned),
    );
    fields.push(format!(
        "\"devDependencies\": {{ \"typescript\": {} }}",
        json_str(TYPESCRIPT)
    ));
    format!("{{\n  {}\n}}\n", fields.join(",\n  "))
}

/// The license's text, by `package.license`: none when the config names none; an error for one
/// the generator has no text of.
fn license(ctx: &Ctx<'_>, issues: &mut Issues) -> Option<String> {
    let named = ctx.config.license.as_ref()?;
    let Some((_, text)) = LICENSES.iter().find(|(id, _)| *id == named.name) else {
        let ids: Vec<&str> = LICENSES.iter().map(|(id, _)| *id).collect();
        issues.error_hint(
            "repo.license",
            &named.at,
            format!("flowapi has no text of the license `{}`", named.name),
            Some(format!(
                "write one of {}, or leave `license` out and add LICENSE yourself",
                ids.join(", ")
            )),
        );
        return None;
    };
    Some(text.replace("{package}", ctx.package()))
}
