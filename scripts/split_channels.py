import json
import os
from os import path

def load_data(data_dir):
    main_file = [file for file in os.listdir(data_dir) if file.endswith('.json')][0]
    with open(path.join(data_dir, main_file), 'r') as f:
        return json.load(f)

def split_channels(data, data_dir):    
    os.makedirs(path.join(data_dir, "categories"), exist_ok=True)
    for i, category in enumerate(data["channels"]["categories"]):
        if not is_public(category):
            continue

        category["children"] = [c for c in category["children"] if is_public(c)]
        if len(category["children"]) == 0:
            continue

        with open(path.join(data_dir, "categories", f"{i}.json"), 'w') as f:
            json.dump(category, f, indent=2)

def split_other_channels(data, data_dir):
    os.makedirs(path.join(data_dir, "other_channels"), exist_ok=True)
    for i, other_channel in enumerate(data["channels"]["others"]):
        if not is_public(other_channel):
            continue

        with open(path.join(data_dir, "other_channels", f"{i}.json"), 'w') as f:
            json.dump(other_channel, f, indent=2)

def is_public(channel):
    permissions_by_name = {p["roleName"]: p for p in channel["permissions"]}
    if "@everyone" not in permissions_by_name:
        return True

    return permissions_by_name["@everyone"]["allow"] != "0"

# Run script from root directory
if __name__ == "__main__":
    data_dir = "./data/final_backup"
    data = load_data(data_dir)
    split_channels(data, data_dir)
    # split_other_channels(data, data_dir)  # Nothing useful in other channels