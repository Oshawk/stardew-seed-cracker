from pathlib import Path

SELF_PATH = Path(__file__)
BASE_PATH = SELF_PATH.parent
INDEX_PATH = BASE_PATH / "index.html"

versions = []
for path in BASE_PATH.iterdir():
    if path.is_dir() and (path / "index.html").is_file():
        versions.append(path.name)

lines = []
for line in INDEX_PATH.read_text().split("\n"):
    if "REPLACE" in line:
        for version in versions:
            lines.append(line.replace("REPLACE", f"""<li><a href="./{version}/">{version}</a></li>"""))
    else:
        lines.append(line)

INDEX_PATH.write_text("\n".join(lines))

SELF_PATH.unlink()
