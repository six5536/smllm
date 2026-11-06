#!/usr/bin/env node
// Build the mdBook source of the documentation site from the pages of docs/.
//
// Usage: node scripts/docs-build.mjs [--tag TAG] [--out DIR]
//
// The script copies each page of docs/ into DIR/src (target/docs/src by
// default), without its frontmatter, and leaves out the schemas and the
// indexes that sokf writes. It turns each link into a link between pages of
// the site, or into a link to the file on GitHub at TAG (main by default), and
// fails when a link leads to a file that doesn't exist. It writes SUMMARY.md:
// the front page, then the tutorials, the how-to guides, the reference, and
// the explanations, each in the order that its pages set. The error pages
// follow the rules page, in the order of its table, and each code of the table
// links its page. `mdbook build` then
// reads book.toml and writes the site into target/docs/html.

import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, posix, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const args = process.argv.slice(2);
const option = (name, fallback) => {
  const at = args.indexOf(name);
  return at === -1 ? fallback : args[at + 1];
};
const tag = option("--tag", "main");
const out = join(root, option("--out", "target/docs"));

/** The public repository, whose files at `tag` a link to a file outside the site leads to. */
const github = "https://github.com/six5536/smllm";

/** The sections of the site, in order, and the page types that each holds. */
const sections = [
  { title: "Tutorials", types: ["Tutorial"] },
  { title: "How-to guides", types: ["HowTo"] },
  { title: "Reference", types: ["Reference", "ErrorReference"] },
  { title: "Explanations", types: ["Explanation"] },
];

// sokf-implements: DOC-2_AC-2
/** Every page of docs/, as repository paths with `/`, without schemas and indexes. */
function pages(dir = "docs") {
  const found = [];
  for (const name of readdirSync(join(root, dir)).sort()) {
    const path = `${dir}/${name}`;
    if (statSync(join(root, path)).isDirectory()) {
      if (path !== "docs/schemas") found.push(...pages(path));
    } else if (name.endsWith(".md") && name !== "index.md") {
      found.push(path);
    }
  }
  return found;
}

/** The frontmatter keys of a page, and its body. */
function split(text) {
  const match = /^---\n([\s\S]*?)\n---\n\n?/.exec(text);
  if (!match) return { meta: {}, body: text, offset: 0 };
  const meta = {};
  for (const line of match[1].split("\n")) {
    const kv = /^([A-Za-z][\w-]*):\s*(.*)$/.exec(line);
    if (kv) meta[kv[1]] = kv[2].replace(/^(["'])(.*)\1$/, "$2");
  }
  return { meta, body: text.slice(match[0].length), offset: match[0].split("\n").length - 1 };
}

const included = new Set(pages());
const problems = [];

// sokf-implements: DOC-2_AC-4
/** The link that `target`, written in the page `page`, has on the site. */
function resolve(page, target, line) {
  if (/^[a-z][a-z0-9+.-]*:/i.test(target) || target.startsWith("#")) return target;
  const [path, fragment] = target.split(/#(.*)/s);
  const repoPath = path.startsWith("/")
    ? path.slice(1)
    : posix.normalize(posix.join(posix.dirname(page), path));
  const hash = fragment === undefined ? "" : `#${fragment}`;
  if (repoPath.startsWith("..") || !existsSync(join(root, repoPath))) {
    problems.push(`${page}:${line}: the link \`${target}\` leads to no file`);
    return target;
  }
  if (included.has(repoPath)) {
    const from = posix.dirname(page.slice("docs/".length));
    return `${posix.relative(from, repoPath.slice("docs/".length)) || "."}${hash}`;
  }
  const kind = statSync(join(root, repoPath)).isDirectory() ? "tree" : "blob";
  return `${github}/${kind}/${tag}/${repoPath.replace(/\/$/, "")}${hash}`;
}

/** A line of prose with each link outside its code spans resolved. */
function rewriteLine(page, text, line) {
  return text
    .split(/(`+[^`]*`+)/)
    .map((part, i) =>
      i % 2 === 1
        ? part
        : part.replace(
            /\]\(([^)\s]+)(\s+"[^"]*")?\)/g,
            (_, target, title = "") => `](${resolve(page, target, line)}${title})`,
          ),
    )
    .join("");
}

/** The body of a page with each link outside a code block resolved. `offset`
 * is the number of lines before the body, so that a problem names the line of
 * the page. */
function rewrite(page, body, offset) {
  let fence = null;
  return body
    .split("\n")
    .map((text, i) => {
      const line = offset + i + 1;
      const opening = /^\s*(`{3,})/.exec(text);
      if (fence) {
        if (opening && text.trim() === fence) fence = null;
        return text;
      }
      if (opening) {
        fence = opening[1];
        return text;
      }
      const definition = /^(\[[^\]]+\]):\s*(\S+)\s*$/.exec(text);
      if (definition) return `${definition[1]}: ${resolve(page, definition[2], line)}`;
      return rewriteLine(page, text, line);
    })
    .join("\n");
}

const files = [];
const chapters = new Map(sections.map((s) => [s.title, []]));
let front = null;
/** The rules page, and the error page of each code. */
let rules = null;
const errors = new Map();
for (const page of included) {
  const { meta, body, offset } = split(readFileSync(join(root, page), "utf8"));
  const file = { dest: join(out, "src", ...page.slice("docs/".length).split("/")), text: rewrite(page, body, offset) };
  files.push(file);
  const link = page.slice("docs/".length);
  if (meta.type === "FrontPage") {
    front = { title: meta.title, link };
    continue;
  }
  if (meta.type === "ErrorReference") {
    errors.set(meta.code, { title: meta.title, link, page });
    continue;
  }
  if (meta.generator === "info-rules") rules = { file, link };
  const section = sections.find((s) => s.types.includes(meta.type));
  if (!section) {
    problems.push(`${page}: the type \`${meta.type}\` belongs to no section of the site`);
    continue;
  }
  chapters.get(section.title).push({ title: meta.title, link, order: Number(meta.order) });
}
if (!front) problems.push("docs/: no page has the type `FrontPage`, which the site opens on");

// sokf-implements: DOC-2_AC-5
/** The codes of the rules table, in its order, each linked to its error page. */
const codes = [];
if (rules) {
  const from = posix.dirname(rules.link);
  rules.file.text = rules.file.text.replace(/^\| `([A-Z]+-[1-9][0-9]*)` \|/gm, (row, code) => {
    codes.push(code);
    const error = errors.get(code);
    return error ? `| [\`${code}\`](${posix.relative(from, error.link)}) |` : row;
  });
}
for (const [code, { page }] of errors) {
  if (!codes.includes(code)) problems.push(`${page}: the code \`${code}\` isn't in the table of the rules page`);
}

// sokf-implements: DOC-2_AC-3
if (problems.length > 0) {
  console.error(problems.join("\n"));
  console.error(`${problems.length} problem(s): the site isn't built`);
  process.exit(1);
}

rmSync(join(out, "src"), { recursive: true, force: true });
for (const { dest, text } of files) {
  mkdirSync(dirname(dest), { recursive: true });
  writeFileSync(dest, text);
}
// sokf-implements: DOC-2_AC-1
const summary = ["# Summary", "", `[${front.title}](${front.link})`, ""];
for (const { title } of sections) {
  const list = chapters.get(title).sort((a, b) => a.order - b.order || a.title.localeCompare(b.title));
  if (list.length === 0) continue;
  const lines = [];
  for (const p of list) {
    lines.push(`- [${p.title}](${p.link})`);
    if (p.link === rules?.link) {
      for (const code of codes.filter((c) => errors.has(c))) {
        const { title, link } = errors.get(code);
        lines.push(`  - [${title}](${link})`);
      }
    }
  }
  summary.push(`# ${title}`, "", ...lines, "");
}
writeFileSync(join(out, "src", "SUMMARY.md"), summary.join("\n"));
console.log(`wrote ${included.size} pages to ${relative(root, join(out, "src")).split(sep).join("/")}`);
