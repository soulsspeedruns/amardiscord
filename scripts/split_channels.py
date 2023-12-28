import json
import os
from os import path

def split_channels(data_dir):
    main_file = [file for file in os.listdir(data_dir) if file.endswith('.json')][0]
    with open(path.join(data_dir, main_file), 'r') as f:
        data = json.load(f)
    
    os.makedirs(path.join(data_dir, "categories"), exist_ok=True)
    for i, category in enumerate(data["channels"]["categories"]):
        with open(path.join(data_dir, "categories", f"{i}.json"), 'w') as f:
            json.dump(category, f, indent=2)

    os.makedirs(path.join(data_dir, "other_channels"), exist_ok=True)
    for i, other_channel in enumerate(data["channels"]["others"]):
        with open(path.join(data_dir, "other_channels", f"{i}.json"), 'w') as f:
            json.dump(other_channel, f, indent=2)


# Run script from root directory
if __name__ == "__main__":
    split_channels("./data/final_backup")