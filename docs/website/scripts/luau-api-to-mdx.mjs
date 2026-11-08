// Makes documentation (mdx files) from the luau type files.
// It contains a simplified Luau parser specifically made to understand type declarations.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

// MARK: Parsing

const OPERATORS = {
	__add: "a + b",
	__sub: "a - b",
	__mul: "a * b",
	__div: "a / b",
	__idiv: "a // b",
	__mod: "a % b",
	__pow: "a ^ b",
	__unm: "-a",
	__len: "#a",
	__eq: "a == b",
	__lt: "a < b",
	__le: "a <= b",
	__concat: "a .. b",
	__call: "a(...)",
	__tostring: "tostring(a)",
};

/** Index of the parenthesis closing the one at `start`, or -1. */
function matchingParen(text, start) {
	let depth = 0;
	for (let i = start; i < text.length; i++) {
		const c = text[i];
		if (c === "(") depth++;
		else if (c === ")") {
			depth--;
			if (depth === 0) return i;
		}
	}
	return -1;
}

/** Splits on commas that are not inside (), {}, [] or <>. */
function splitTopLevel(text, separator = ",") {
	const parts = [];
	let depth = 0;
	let current = "";
	let previous = "";
	for (const c of text) {
		if ("({[<".includes(c)) depth++;
		// The `>` of a function type arrow `->` is not a closing bracket.
		else if (")}]".includes(c) || (c === ">" && previous !== "-")) depth--;
		previous = c;
		if (c === separator && depth === 0) {
			parts.push(current);
			current = "";
		} else current += c;
	}
	if (current.trim() !== "") parts.push(current);
	return parts.map((p) => p.replace(/\s+/g, " ").trim()).filter((p) => p !== "");
}

/** Parses `function a.b<T>(x: number, y: T): R` (possibly spanning several lines). */
function parseSignature(text) {
	const flat = text.replace(/\s+/g, " ");
	const head = flat.match(/^(?:local\s+)?function\s+([\w.:]+)\s*(<[^(]*>)?\s*\(/) ?? flat.match(/=\s*function\s*(<[^(]*>)?\s*\(/);
	if (!head) return null;
	const open = flat.indexOf("(", head.index + head[0].length - 1);
	const close = matchingParen(flat, open);
	const params = splitTopLevel(flat.slice(open + 1, close));
	let ret = flat.slice(close + 1).trim();
	ret = ret.startsWith(":") ? ret.slice(1).trim() : "";
	ret = ret.replace(/\s*\bend$/, "");
	return { fullName: head[0].startsWith("=") ? null : head[1], generics: head[0].startsWith("=") ? head[1] ?? "" : head[2] ?? "", params, ret };
}

/** Splits doc lines into description markdown and @tags. */
function parseDoc(lines) {
	const description = [];
	const params = [];
	let returns = "";
	// The tag whose text the next lines continue: a tag's text can wrap onto several lines, until a blank line,
	// another tag or a `#` section.
	let continuing = null;
	for (const line of lines) {
		const tag = line.match(/^\s*@(\w+)\s*(.*)$/);
		if (!tag) {
			if (continuing && line.trim() !== "" && !/^\s*#/.test(line)) {
				if (continuing === "return") returns += " " + line.trim();
				else continuing.text += " " + line.trim();
				continue;
			}
			continuing = null;
			description.push(line);
			continue;
		}
		const [, name, rest] = tag;
		continuing = null;
		if (name === "param") {
			const m = rest.match(/^(\S+)\s*(.*)$/);
			if (m) {
				const param = { name: m[1], text: m[2].replace(/^[—-]\s*/, "").trim() };
				params.push(param);
				continuing = param;
			}
		} else if (name === "return" || name === "returns") {
			returns = rest.trim();
			continuing = "return";
		}
	}
	while (description.length && description[0].trim() === "") description.shift();
	while (description.length && description.at(-1).trim() === "") description.pop();
	return { description: description.join("\n"), params, returns };
}

/**
 * Returns the fields of the first `{ ... }` table in a type body that have a `---` doc comment above them.
 * Undocumented fields are not returned: a type's full definition is not shown, only what the author documented.
 */
function documentedFields(body) {
	const open = body.indexOf("{");
	if (open === -1) return [];
	const fields = [];
	let doc = [];
	let depth = 0;
	let field = null;
	for (const rawLine of body.slice(open + 1).split("\n")) {
		const line = rawLine.trim();
		if (depth === 0 && !field && line.startsWith("}")) break;
		if (depth === 0 && !field && line.startsWith("---")) {
			doc.push(line.replace(/^---\s?/, ""));
			continue;
		}
		if (depth === 0 && !field && (line === "" || line.startsWith("--"))) continue;
		if (!field) {
			const m = line.match(/^(\[[^\]]+\]|\w+)\s*:\s*(.*)$/);
			if (!m) {
				doc = [];
				continue;
			}
			field = { name: m[1], type: m[2], doc };
			doc = [];
		} else {
			field.type += " " + line;
		}
		for (const c of line.replace(/->/g, "")) {
			if ("({[".includes(c)) depth++;
			else if (")}]".includes(c)) depth--;
		}
		if (depth <= 0) {
			depth = 0;
			field.type = field.type.replace(/,\s*$/, "").replace(/\s+/g, " ").trim();
			if (field.doc.some((l) => l.trim())) fields.push({ name: field.name, type: field.type, doc: parseDoc(field.doc) });
			field = null;
		}
	}
	return fields;
}

/** Collects a declaration starting at line `i` until its parentheses are balanced. Returns [text, nextIndex]. */
function collectSignature(lines, i) {
	let text = lines[i];
	let depth = 0;
	let started = false;
	let j = i;
	for (;;) {
		for (const c of lines[j]) {
			if (c === "(") {
				depth++;
				started = true;
			} else if (c === ")") depth--;
		}
		if (started && depth <= 0) break;
		j++;
		if (j >= lines.length) break;
		text += "\n" + lines[j];
	}
	return [text, j + 1];
}

/** Skips a function body: everything up to and including the next `end` at the start of a line. */
const DECLARATION = /^(function |local function |export type |type |module\.\w+\s*=|---)/;

/**
 * Skips a function body, up to and including the `end` that closes it. `i` is the line after the signature.
 * A body written on the signature line (`function f(): () end`) has nothing to skip.
 */
function skipBody(lines, i, warn) {
	if (/\bend\s*$/.test(lines[i - 1])) return i;
	let swallowed = false;
	while (i < lines.length && lines[i] !== "end") {
		if (!swallowed && DECLARATION.test(lines[i])) {
			warn(i, "this declaration is inside the body of the previous function (is its `end` missing?), so it is not documented");
			swallowed = true;
		}
		i++;
	}
	if (i >= lines.length) warn(i - 1, "a function body is never closed by an `end` at the start of a line");
	return i + 1;
}

/** Index after a statement that opens brackets on line `i` and closes them on later lines. */
function skipBrackets(lines, i) {
	let depth = 0;
	let j = i;
	do {
		for (const c of lines[j]) {
			if ("({[".includes(c)) depth++;
			else if (")}]".includes(c)) depth--;
		}
		j++;
	} while (depth > 0 && j < lines.length);
	return j;
}

/** Top-level lines that are expected and have nothing to document. */
const IGNORED = [/^local \w+\s*=\s*require\(/, /^const \w+\s*=\s*require\(/, /^(\w+)\.__index\s*=\s*\1\s*$/, /^return module\s*$/];

/** Collects a type declaration: the first line, then every indented or continuation line. */
function collectType(lines, i) {
	const parts = [];
	let depth = 0;
	let j = i;
	while (j < lines.length) {
		const line = lines[j];
		if (j > i && depth <= 0 && (line === "" || !/^(\s|[|&}\])])/.test(line))) break;
		for (const c of line.replace(/--.*$/, "")) {
			if ("({[".includes(c)) depth++;
			else if (")}]".includes(c)) depth--;
		}
		parts.push(line);
		j++;
	}
	return [parts.join("\n"), j];
}

/** `sourceLabel` identifies the file in warnings, for example "luau-api/vec.luau (master)". */
export function parseModule(source, moduleName, sourceLabel = `${moduleName}.luau`) {
	const warn = (lineIndex, message) => console.warn(`warning: ${sourceLabel}:${lineIndex + 1}: ${message}`);
	const lines = source.replace(/\r\n/g, "\n").split("\n");
	const mod = { name: moduleName, description: "", functions: [], classes: [], types: [], constants: [] };
	const implToClass = new Map();
	const implDocs = new Map();
	const methodsByImpl = new Map();
	const operatorsByImpl = new Map();
	const globalFunctions = new Map();
	const typedLocals = new Map();
	let doc = [];

	const addToImpl = (map, impl, item) => {
		if (!map.has(impl)) map.set(impl, []);
		map.get(impl).push(item);
	};

	let i = 0;
	while (i < lines.length) {
		const line = lines[i];

		if (line.startsWith("--[[")) {
			const block = [];
			let j = i;
			const first = line.slice(4);
			if (first.includes("]]")) block.push(first.slice(0, first.indexOf("]]")));
			else {
				if (first.trim()) block.push(first);
				j++;
				while (j < lines.length && !lines[j].includes("]]")) block.push(lines[j++]);
				if (j < lines.length) block.push(lines[j].slice(0, lines[j].indexOf("]]")));
			}
			doc = block;
			i = j + 1;
			continue;
		}
		if (line.startsWith("---")) {
			doc.push(line.replace(/^---\s?/, ""));
			i++;
			continue;
		}
		if (line.startsWith("--") || line.trim() === "") {
			i++;
			continue;
		}

		if (/^(local|const) module\s*=\s*\{\s*\}/.test(line)) {
			mod.description = parseDoc(doc).description;
			doc = [];
			i++;
			continue;
		}

		let m;
		if ((m = line.match(/^local (\w+)\s*=\s*\{/))) {
			if (doc.length) implDocs.set(m[1], parseDoc(doc));
			doc = [];
			i = skipBrackets(lines, i);
			continue;
		}

		if ((m = line.match(/^(export )?type (\w+)(<[^=]*>)?\s*=/))) {
			const [text, next] = collectType(lines, i);
			const body = text.slice(text.indexOf("=") + 1).replace(/\s--(?!-)[^\n]*$/gm, "").trim();
			const parsedDoc = parseDoc(doc);
			const classMatch = body.match(/typeof\(\s*setmetatable\(\s*\{\}\s*(?:::[^,]*)?,\s*(\w+)\s*\)\s*\)/);
			if (classMatch) {
				implToClass.set(classMatch[1], {
					name: m[2],
					generics: m[3] ?? "",
					exported: Boolean(m[1]),
					doc: parsedDoc,
					impl: classMatch[1],
				});
			} else {
				mod.types.push({ name: m[2], generics: m[3] ?? "", exported: Boolean(m[1]), doc: parsedDoc, fields: documentedFields(body) });
			}
			doc = [];
			i = next;
			continue;
		}

		if ((m = line.match(/^(local )?function ([\w.:]+)/))) {
			const [text, afterSignature] = collectSignature(lines, i);
			const sig = parseSignature(text);
			const parsedDoc = parseDoc(doc);
			doc = [];
			i = skipBody(lines, afterSignature, warn);
			if (!sig || m[1]) continue;
			const fullName = sig.fullName;
			const methodMatch = fullName.match(/^(\w+)([.:])(\w+)$/);
			if (!methodMatch) {
				globalFunctions.set(fullName, { name: fullName, ...sig, doc: parsedDoc });
				continue;
			}
			const [, owner, separator, name] = methodMatch;
			const item = { name, generics: sig.generics, params: sig.params, ret: sig.ret, doc: parsedDoc, colon: separator === ":" };
			if (owner === "module") mod.functions.push(item);
			else if (name.startsWith("__") && OPERATORS[name]) addToImpl(operatorsByImpl, owner, item);
			else addToImpl(methodsByImpl, owner, item);
			continue;
		}

		if ((m = line.match(/^local (\w+)\s*:\s*(.+?)\s*=\s*function/))) {
			typedLocals.set(m[1], m[2]);
			const [, afterSignature] = collectSignature(lines, i);
			i = skipBody(lines, afterSignature, warn);
			doc = [];
			continue;
		}

		if ((m = line.match(/^(?!module\.)(\w+)\.(__\w+)\s*=\s*(.*)$/))) {
			const [, impl, name, rhs] = m;
			const parsedDoc = parseDoc(doc);
			doc = [];
			if (rhs.startsWith("function")) {
				const [text, afterSignature] = collectSignature(lines, i);
				const sig = parseSignature(text);
				i = skipBody(lines, afterSignature, warn);
				if (sig && OPERATORS[name]) addToImpl(operatorsByImpl, impl, { name, generics: sig.generics, params: sig.params, ret: sig.ret, doc: parsedDoc });
				continue;
			}
			if (OPERATORS[name] && typedLocals.has(rhs.trim())) {
				addToImpl(operatorsByImpl, impl, { name, typeText: typedLocals.get(rhs.trim()), doc: parsedDoc });
			}
			i++;
			continue;
		}

		if ((m = line.match(/^module\.(\w+)\s*=\s*(.+)$/))) {
			const [, name, rhs] = m;
			const parsedDoc = parseDoc(doc);
			doc = [];
			const value = rhs.replace(/\s--(?!-).*$/, "").trim();
			if (globalFunctions.has(value)) {
				const fn = globalFunctions.get(value);
				mod.functions.push({ name, generics: fn.generics, params: fn.params, ret: fn.ret, doc: parsedDoc.description || parsedDoc.params.length ? parsedDoc : fn.doc });
			} else {
				mod.constants.push({ name, value, doc: parsedDoc });
			}
			i++;
			continue;
		}

		if (!IGNORED.some((pattern) => pattern.test(line))) {
			warn(i, `unrecognized line, not documented: ${line.trim()}`);
		}
		doc = [];
		i = skipBrackets(lines, i);
	}

	// Methods are never dropped: an Impl table whose type was not found still gets a class, named after the Impl.
	for (const impl of new Set([...methodsByImpl.keys(), ...operatorsByImpl.keys()])) {
		if (implToClass.has(impl)) continue;
		const name = impl.replace(/Impl$/, "");
		console.warn(`warning: ${sourceLabel}: no \`export type X = typeof(setmetatable(..., ${impl}))\` found, documenting its methods as \`${name}\``);
		implToClass.set(impl, { name, generics: "", exported: false, doc: { description: "", params: [], returns: "" }, impl });
	}

	for (const [impl, cls] of implToClass) {
		if (!cls.doc.description && implDocs.has(impl)) cls.doc = implDocs.get(impl);
		cls.methods = methodsByImpl.get(impl) ?? [];
		cls.operators = operatorsByImpl.get(impl) ?? [];
		mod.classes.push(cls);
	}
	return mod;
}

// MARK: Links

/** Same slugs as the heading anchors Starlight generates (github-slugger). */
function slug(text) {
	return text
		.toLowerCase()
		.replace(/[^\p{L}\p{N}\s_-]/gu, "")
		.replace(/\s/g, "-");
}

/** Separator used in a method's heading: `:` for methods called on a value, `.` otherwise. Returns [separator, params]. */
function methodShape(method, className) {
	if (method.colon) return [":", method.params];
	if (method.params.length && isClassValue(method.params[0], className)) return [":", method.params.slice(1)];
	return [".", method.params];
}

/** Set while rendering a version: every linkable name of that version, and the module being rendered. */
let links = null;

function buildLinks(modules, basePath) {
	const anchors = new Map();
	const types = new Map();
	const addType = (mod, name, url) => {
		anchors.set(`${mod}.${name}`, url);
		anchors.set(`${mod.charAt(0).toUpperCase() + mod.slice(1)}.${name}`, url);
		if (!types.has(name)) types.set(name, new Map());
		types.get(name).set(mod, url);
	};
	for (const mod of modules) {
		const page = `${basePath}/${mod.name}/`;
		anchors.set(mod.name, page);
		for (const fn of mod.functions) anchors.set(`${mod.name}.${fn.name}`, page + "#" + slug(`${mod.name}.${fn.name}`));
		for (const cls of mod.classes) {
			addType(mod.name, cls.name, page + "#" + slug(cls.name));
			for (const method of cls.methods) {
				const [separator] = methodShape(method, cls.name);
				const url = page + "#" + slug(`${cls.name}${separator}${method.name}`);
				anchors.set(`${cls.name}:${method.name}`, url);
				anchors.set(`${cls.name}.${method.name}`, url);
			}
		}
		for (const type of mod.types) {
			addType(mod.name, type.name, page + "#" + slug(type.name));
			for (const field of type.fields ?? []) anchors.set(`${type.name}.${field.name}`, page + "#" + slug(`${type.name}.${field.name}`));
		}
		for (const c of mod.constants) anchors.set(`${mod.name}.${c.name}`, page + "#" + slug(`${mod.name}.${c.name}`));
	}
	return { anchors, types, modules: new Map(modules.map((m) => [m.name, m])), current: null };
}

/** `code` is an inline code span with its backticks. Returns it wrapped in a link when it names something of the reference. */
function linkCode(code) {
	if (!links) return code;
	const key = code.slice(1, -1).trim().replace(/\(.*\)$/, "");
	let url = links.anchors.get(key);
	const current = links.current;
	if (!url && current && /^\w+$/.test(key)) {
		if (links.modules.get(current)?.functions.some((fn) => fn.name === key)) url = links.anchors.get(`${current}.${key}`);
		else if (links.types.has(key)) {
			const byModule = links.types.get(key);
			url = byModule.get(current) ?? (byModule.size === 1 ? [...byModule.values()][0] : undefined);
		}
	}
	return url ? `[${code}](${url})` : code;
}

// MARK: Rendering

/** Escapes characters that MDX would read as JSX or expressions, outside of code. */
function escapeMdx(markdown) {
	return markdown
		.split(/(```[\s\S]*?```)/g)
		.map((part, index) => {
			if (index % 2 === 1) return part.replace(/^```lua\b/, "```luau").replace(/^```\s*\n/, "```luau\n");
			return part
				.split(/(`[^`\n]*`)/g)
				.map((piece, j) => (j % 2 === 1 ? linkCode(piece) : piece.replace(/[{}]/g, (c) => "\\" + c).replace(/</g, "&lt;")))
				.join("");
		})
		.join("");
}

/** Docs may contain `# headings`: they are moved below the heading of the page (by 2) or of the item they document (by 3). */
function demoteHeadings(markdown, levels = 2) {
	return markdown.replace(/^(#{1,4}) /gm, (_, hashes) => "#".repeat(Math.min(6, hashes.length + levels)) + " ");
}

function itemMarkdown(markdown) {
	return demoteHeadings(escapeMdx(markdown), 3);
}

function renderSignature(prefix, item, params) {
	const ret = item.ret ? `: ${item.ret}` : "";
	const oneLine = `${prefix}${item.generics ?? ""}(${params.join(", ")})${ret}`;
	if (oneLine.length <= 90 || params.length === 0) return oneLine;
	return `${prefix}${item.generics ?? ""}(\n${params.map((p) => `\t${p},`).join("\n")}\n)${ret}`;
}

function codeBlock(code) {
	return "```luau showLineNumbers=false\n" + code + "\n```";
}

function renderDocBody(doc, signatureParams = []) {
	const types = new Map();
	for (const param of signatureParams) {
		const m = param.match(/^([\w.]+)\s*:\s*(.+)$/);
		if (m) types.set(m[1], m[2]);
	}
	// The overview comes first, then parameters and return value, then the `# Errors` / `# Example` sections.
	const lines = doc.description.split("\n");
	let inFence = false;
	let sectionStart = lines.length;
	for (let i = 0; i < lines.length; i++) {
		if (lines[i].trim().startsWith("```")) inFence = !inFence;
		if (!inFence && /^#{1,4} /.test(lines[i])) {
			sectionStart = i;
			break;
		}
	}
	const overview = lines.slice(0, sectionStart).join("\n").trim();
	const sections = lines.slice(sectionStart).join("\n").trim();
	const out = [];
	if (overview) out.push(itemMarkdown(overview));
	if (doc.params.length) {
		const rows = doc.params.map((p) => {
			const type = types.get(p.name);
			return `- \`${p.name}\`${type ? ` (${linkCode("`" + type + "`")})` : ""}${p.text ? `: ${escapeMdx(p.text)}` : ""}`;
		});
		out.push("**Parameters**\n\n" + rows.join("\n"));
	}
	if (doc.returns) out.push(`**Returns** ${escapeMdx(doc.returns)}`);
	if (sections) out.push(itemMarkdown(sections));
	return out.join("\n\n");
}

function renderFunction(heading, signature, doc, params) {
	const parts = [`### ${heading}`, codeBlock(signature)];
	const body = renderDocBody(doc, params);
	if (body) parts.push(body);
	return parts.join("\n\n");
}

function isClassValue(param, className) {
	const m = param.match(/^(\w+)\s*:\s*(.+)$/);
	if (!m) return param === "self";
	return m[1] === "self" || m[2].replace(/<.*>$/, "") === className;
}

function typeDeclaration(type) {
	return codeBlock(`${type.exported ? "export " : ""}type ${type.name}${type.generics} = ...`);
}

function renderClass(cls) {
	const parts = [`## ${cls.name}`, typeDeclaration(cls)];
	if (cls.doc.description) parts.push(itemMarkdown(cls.doc.description));

	if (cls.operators.length) {
		parts.push("**Operators**");
		for (const op of cls.operators) {
			const types = op.typeText ? ` — \`${op.typeText}\`` : op.params ? ` — \`(${op.params.join(", ")})${op.ret ? `: ${op.ret}` : ""}\`` : "";
			parts.push(`\`${OPERATORS[op.name]}\`${types}`);
			if (op.doc.description) parts.push(itemMarkdown(op.doc.description));
		}
	}

	for (const method of cls.methods) {
		const [separator, params] = methodShape(method, cls.name);
		const heading = `${cls.name}${separator}${method.name}`;
		parts.push(renderFunction(heading, renderSignature(heading, method, params), method.doc, params));
	}
	return parts.join("\n\n");
}

function renderType(type) {
	const parts = [`### ${type.name}`, typeDeclaration(type)];
	if (type.doc.description) parts.push(itemMarkdown(type.doc.description));
	for (const field of type.fields ?? []) {
		parts.push(`#### ${type.name}.${field.name}`, codeBlock(`${field.name}: ${field.type}`));
		const body = renderDocBody(field.doc);
		if (body) parts.push(demoteHeadings(body, 1));
	}
	return parts.join("\n\n");
}

function firstSentence(markdown) {
	const text = markdown
		.split("\n")
		.filter((l) => l.trim() && !l.startsWith("#"))
		.join(" ")
		.trim();
	const m = text.match(/^(.+?\.)(\s|$)/);
	return (m ? m[1] : text).trim();
}

function frontmatterString(value) {
	return JSON.stringify(value);
}

function versionSelect(options, slug) {
	const { version, versionLabel, latestPath, masterPath, compareModules } = options;
	const other = version === "master" ? latestPath : masterPath;
	const otherHref = slug && (!compareModules || compareModules.has(slug)) ? `${other}/${slug}/` : `${other}/`;
	const latestHref = version === "latest" ? (slug ? `${latestPath}/${slug}/` : `${latestPath}/`) : otherHref;
	const masterHref = version === "master" ? (slug ? `${masterPath}/${slug}/` : `${masterPath}/`) : otherHref;
	return `<ReferenceVersionSelect current="${version}" latestLabel="${versionLabel}" latestHref="${latestHref}" masterHref="${masterHref}" />`;
}

function pageHeader(options, title, description, slug, order) {
	const isMaster = options.version === "master";
	const frontmatter = ["---", `title: ${frontmatterString(title)}`, `description: ${frontmatterString(description)}`];
	if (isMaster) frontmatter.push("sidebar:", "    hidden: true", "prev: false", "next: false");
	else frontmatter.push("sidebar:", `    order: ${order}`, ...(slug ? [] : ["    label: Overview"]));
	frontmatter.push("---");
	const lines = [
		frontmatter.join("\n"),
		"",
		"{/* Generated by scripts/luau-api-to-mdx.mjs from the luau-api folder. Do not edit by hand: run `pnpm references`. */}",
		"",
		'import { Aside } from "@astrojs/starlight/components";',
		`import ReferenceVersionSelect from "${isMaster ? "../../../../" : "../../../"}components/ReferenceVersionSelect.astro";`,
		"",
		versionSelect(options, slug),
	];
	if (isMaster) {
		lines.push(
			"",
			`<Aside type="caution">This is the reference of the unreleased master version of Vectarine. Some of these features may not be available in the latest release available on GitHub (${options.versionLabel}).</Aside>`,
		);
	}
	return lines.join("\n");
}

export function renderModulePage(mod, options, order) {
	if (links) links.current = mod.name;
	const title = mod.name.charAt(0).toUpperCase() + mod.name.slice(1);
	const description = firstSentence(mod.description) || `Reference of the ${mod.name} module.`;
	const parts = [pageHeader(options, title, description, mod.name, order)];
	parts.push(codeBlock(`const ${mod.name} = require("@vectarine/${mod.name}")`));
	if (mod.description) parts.push(demoteHeadings(escapeMdx(mod.description)));

	if (mod.functions.length) {
		parts.push("## Functions");
		for (const fn of mod.functions) {
			const heading = `${mod.name}.${fn.name}`;
			parts.push(renderFunction(heading, renderSignature(heading, fn, fn.params), fn.doc, fn.params));
		}
	}

	for (const cls of mod.classes) parts.push(renderClass(cls));

	if (mod.types.length) {
		parts.push("## Types");
		for (const type of mod.types) parts.push(renderType(type));
	}

	if (mod.constants.length) {
		parts.push("## Constants");
		for (const c of mod.constants) {
			const name = `${mod.name}.${c.name}`;
			parts.push(`### ${name}`, codeBlock(`${name} = ...`));
			if (c.doc.description) parts.push(itemMarkdown(c.doc.description));
		}
	}
	return parts.join("\n\n") + "\n";
}

export function renderIndexPage(modules, options) {
	if (links) links.current = null;
	const isMaster = options.version === "master";
	const title = isMaster ? "Reference (master)" : "Reference";
	const parts = [pageHeader(options, title, "The list of all the modules of Vectarine, with their functions, types and constants.", null, 0)];
	parts.push(
		"This reference lists everything Vectarine provides to your Luau code. It is generated from the `luau-api` folder that is inside every Vectarine project, so you can also read it directly in your editor.",
	);
	parts.push("To use a module, import it with `require`, for example `const vec = require(\"@vectarine/vec\")`.");
	parts.push("## Modules");
	const base = isMaster ? options.masterPath : options.latestPath;
	parts.push(
		modules
			.map((mod) => {
				const summary = firstSentence(mod.description);
				return `- [\`${mod.name}\`](${base}/${mod.name}/)${summary ? `: ${escapeMdx(summary)}` : ""}`;
			})
			.join("\n"),
	);
	return parts.join("\n\n") + "\n";
}

// MARK: Generation

export function listModules(inputDir) {
	return fs
		.readdirSync(inputDir)
		.filter((f) => f.endsWith(".luau"))
		.map((f) => f.slice(0, -".luau".length))
		.sort();
}

/**
 * Generates every page of one version into outputDir (which is emptied first, except for subfolders listed in keep).
 * options: { version: "latest" | "master", versionLabel, latestPath, masterPath, compareModules?: Set<string>, keep?: string[] }
 */
export function generateReference(inputDir, outputDir, options) {
	fs.mkdirSync(outputDir, { recursive: true });
	for (const entry of fs.readdirSync(outputDir)) {
		if ((options.keep ?? []).includes(entry)) continue;
		fs.rmSync(path.join(outputDir, entry), { recursive: true, force: true });
	}
	const versionName = options.version === "master" ? "master" : options.versionLabel;
	const modules = listModules(inputDir).map((name) =>
		parseModule(fs.readFileSync(path.join(inputDir, `${name}.luau`), "utf8"), name, `luau-api/${name}.luau (${versionName})`),
	);
	links = buildLinks(modules, options.version === "master" ? options.masterPath : options.latestPath);
	modules.forEach((mod, index) => {
		fs.writeFileSync(path.join(outputDir, `${mod.name}.mdx`), renderModulePage(mod, options, index + 1));
	});
	fs.writeFileSync(path.join(outputDir, "index.mdx"), renderIndexPage(modules, options));
	links = null;
	return modules;
}

function parseArgs(argv) {
	const args = {};
	for (let i = 0; i < argv.length; i += 2) args[argv[i].replace(/^--/, "")] = argv[i + 1];
	return args;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
	const args = parseArgs(process.argv.slice(2));
	if (!args.input || !args.output) {
		console.error("Usage: node scripts/luau-api-to-mdx.mjs --input <luau-api dir> --output <dir> [--version latest|master] [--version-label 0.7.0]");
		process.exit(1);
	}
	const modules = generateReference(args.input, args.output, {
		version: args.version ?? "latest",
		versionLabel: args["version-label"] ?? "latest",
		latestPath: args["latest-path"] ?? "/references",
		masterPath: args["master-path"] ?? "/references/master",
		compareModules: args.compare ? new Set(listModules(args.compare)) : undefined,
	});
	console.log(`Generated ${modules.length} module pages in ${args.output}`);
}
