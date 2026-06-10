# Ixeo Frontend Client

This directory contains the raw HTML, CSS, and JavaScript source files for the Ixeo search frontend. 

To maximize performance, these files are not served directly. Instead, they are minified, compressed, and output into the `http-server/dist` directory, where the Rust Axum server embeds them directly into its binary at compile time.

## Prerequisites

You must have [Node.js and npm](https://nodejs.org/) installed to run the build tools.

## Installation

Clone the repository:

```bash
git clone https://github.com/hypenexy/ixeo.git
```

Navigate into the directory and install the required dependencies (`html-minifier-terser`, `clean-css-cli`, and `terser`):

```bash
cd html-client
npm install
```

## Building for Production

Whenever you make changes to `index.html`, `style.css`, or `script.js`, you must rebuild the frontend before compiling the Rust server.

Run the following command:

```bash
npm run build
```

### What this does:
1. **HTML:** Strips comments and collapses whitespace.
2. **CSS:** Performs advanced minification and structural optimization.
3. **JS:** Compresses logic and mangles variable names.
4. Outputs the production-ready files into `../http-server/dist/`.

**Important:** Because the Rust server uses `include_str!` to load these files into memory, you **must** run `npm run build` *before* running `cargo build` on the backend if you want your frontend changes to appear!