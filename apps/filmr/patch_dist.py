import os
import glob

DIST_DIR = os.environ['TRUNK_STAGING_DIR']

def patch_worker_helpers():
    print("Searching for workerHelpers.js...")
    helpers_files = glob.glob(os.path.join(DIST_DIR, "snippets", "**", "workerHelpers.js"), recursive=True)
    
    if not helpers_files:
        print("Error: workerHelpers.js not found!")
        return

    for file_path in helpers_files:
        print(f"Patching {file_path}...")
        with open(file_path, "r") as f:
            content = f.read()
        
        # Replace the buggy relative import with absolute path to main JS
        new_content = content.replace("import('../../..')", "import(new URL('../../../filmr_app.js', import.meta.url).href)")
        
        if content == new_content:
            print("  - Already patched or pattern not found.")
        else:
            with open(file_path, "w") as f:
                f.write(new_content)
            print("  - Patch applied.")

def patch_index_html():
    print("Patching index.html...")
    index_path = os.path.join(DIST_DIR, "index.html")
    
    if not os.path.exists(index_path):
        print("Error: index.html not found!")
        return

    with open(index_path, "r") as f:
        content = f.read()

    # Remove the modulepreload link for workerHelpers.js to avoid integrity errors
    # Pattern: <link rel="modulepreload" href="/snippets/.../workerHelpers.js" ...>
    import re
    # Regex to match the full link tag containing workerHelpers.js
    pattern = r'<link rel="modulepreload" href="[^"]*workerHelpers\.js"[^>]*>'
    
    new_content = re.sub(pattern, '', content)
    
    if content == new_content:
        print("  - No problematic modulepreload link found (or already removed).")
    else:
        with open(index_path, "w") as f:
            f.write(new_content)
        print("  - Removed problematic modulepreload link.")

if __name__ == "__main__":
    patch_worker_helpers()
    patch_index_html()
    print("Done.")
