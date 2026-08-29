# Repository Guidelines

## Project Structure & Module Organization
- `src/routes/` contains SvelteKit route pages and endpoints.
- `src/lib/` holds shared components, utilities, and client-only helpers.
- `src/app.html`, `src/app.css`, and `src/app.d.ts` are app shell, global styles, and typings.
- `static/` is for public assets served as-is (e.g. `static/robots.txt`).
- Build output lands in `build/` when using the static adapter.

## Build, Test, and Development Commands
- `pnpm dev`: run the Vite dev server with HMR.
- `pnpm build`: create a production build via SvelteKit.
- `pnpm preview`: serve the production build locally.
- `pnpm check` / `pnpm check:watch`: run `svelte-check` for type and Svelte diagnostics.
- `pnpm format`: format the repository with Prettier.
- `pnpm lint`: verify formatting (Prettier check only).

## Coding Style & Naming Conventions
- Indentation uses tabs; single quotes; trailing commas disabled; `printWidth` 100.
- Svelte files are formatted with `prettier-plugin-svelte` and Tailwind class sorting via `prettier-plugin-tailwindcss`.
- Keep filenames and routes lowercase; match SvelteKit conventions in `src/routes` (e.g. `+page.svelte`).

## Testing Guidelines
- No automated test runner is configured. Use `pnpm check` to validate types and Svelte files.
- If adding tests, align with SvelteKit conventions and keep them close to relevant modules in `src/`.

## Commit & Pull Request Guidelines
- Commit message convention is not defined; recent history uses emoji-only subjects (e.g. `🙃`).
- Prefer short, descriptive summaries unless the team specifies otherwise.
- Pull requests should include a clear description, linked issues, and screenshots for UI changes.

## Configuration Notes
- Package manager: `pnpm` (see `pnpm-lock.yaml`).
- Framework: SvelteKit with the static adapter; Vite handles build and dev tooling.
