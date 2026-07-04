import sqlite3
from pathlib import Path

# Paths
db_path = "content.sqlite3"
image_path = Path("images/vehicles/2017-F151M.png")

# Read image as bytes
image_bytes = image_path.read_bytes()

# Connect to SQLite
conn = sqlite3.connect(db_path)
cursor = conn.cursor()

# Update blob
cursor.execute(
    """
    UPDATE vehicles
    SET image = ?
    WHERE year = ? AND vehicle = ?
    """,
    (image_bytes, 2017, "F151M")
)

conn.commit()
conn.close()

