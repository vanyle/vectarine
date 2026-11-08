// Generates the reference pages for the two versions shown on the website:
//   - the latest release (the highest x.y.z git tag), in src/content/docs/references/
//   - master (the luau-api folder of the working tree), in src/content/docs/references/master/
// Run with `pnpm references`. `pnpm build` runs it too, and CI fails if the committed pages are not up to date.

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { generateReference, listModules } from "./luau-api-to-mdx.mjs";

const websiteDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repoDir = path.resolve(websiteDir, "..", "..");
const masterInput = path.join(repoDir, "luau-api");
const outputDir = path.join(websiteDir, "src", "content", "docs", "references");
const latestPath = "/references";
const masterPath = "/references/master";

const git = (...args) => execFileSync("git", args, { cwd: repoDir, encoding: "utf8" });

function latestReleaseTag() {
	const tags = git("tag", "--list")
		.split("\n")
		.map((t) => t.trim())
		.filter((t) => /^\d+\.\d+\.\d+$/.test(t));
	if (tags.length === 0) {
		throw new Error("No x.y.z git tag found. Fetch the latest release tag first (see the 'Fetch the latest release tag' step in .github/workflows/docs.yml).");
	}
	const parse = (t) => t.split(".").map(Number);
	tags.sort((a, b) => {
		const [x, y] = [parse(a), parse(b)];
		return x[0] - y[0] || x[1] - y[1] || x[2] - y[2];
	});
	return tags.at(-1);
}

/** Writes the luau-api folder as it was at `tag` into a temporary folder. */
function extractLuauApi(tag) {
	const dir = fs.mkdtempSync(path.join(os.tmpdir(), `luau-api-${tag}-`));
	const files = git("ls-tree", "--name-only", `${tag}:luau-api`)
		.split("\n")
		.filter((f) => f.endsWith(".luau"));
	for (const file of files) {
		fs.writeFileSync(path.join(dir, file), git("show", `${tag}:luau-api/${file}`));
	}
	return dir;
}

const tag = latestReleaseTag();
const latestInput = extractLuauApi(tag);
try {
	const latestModules = new Set(listModules(latestInput));
	const masterModules = new Set(listModules(masterInput));
	const shared = { versionLabel: tag, latestPath, masterPath };

	const latest = generateReference(latestInput, outputDir, { ...shared, version: "latest", compareModules: masterModules, keep: ["master"] });
	const master = generateReference(masterInput, path.join(outputDir, "master"), { ...shared, version: "master", compareModules: latestModules });
	console.log(`Reference generated: ${latest.length} modules for ${tag}, ${master.length} modules for master.`);
} finally {
	fs.rmSync(latestInput, { recursive: true, force: true });
}
