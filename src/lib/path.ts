import { homeDir } from "@tauri-apps/api/path";

let home: string | undefined;

async function getHome(): Promise<string> {
	if (home === undefined) {
		home = await homeDir().catch(() => "");
	}
	return home;
}

export async function displayPath(path: string): Promise<string> {
	const h = await getHome();
	if (!h || !path.startsWith(h)) return path;
	const rest = path
		.slice(h.length)
		.replace(/^[\\/]/, "")
		.replaceAll("\\", "/");
	return `~/${rest}`;
}

/**
 * An absolute path to `relative` (a repo-relative path, always written with `/`
 * by git) under `base`, using the separator `base` itself uses.
 */
export function joinPath(base: string, relative: string): string {
	const sep = base.includes("\\") && !base.includes("/") ? "\\" : "/";
	const tail = sep === "\\" ? relative.replaceAll("/", "\\") : relative;
	return base.endsWith(sep) ? `${base}${tail}` : `${base}${sep}${tail}`;
}

/** The last segment of a path, split on either separator (`C:\a\b` and `/a/b` both give `b`). */
export function pathBasename(path: string): string {
	return path.split(/[\\/]/).filter(Boolean).at(-1) ?? path;
}

/** Everything before the last segment, with a trailing separator ("" when there is none). */
export function pathParent(path: string): string {
	const cut = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
	return cut < 0 ? "" : path.slice(0, cut + 1);
}
