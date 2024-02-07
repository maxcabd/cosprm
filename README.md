# cosprm
A tool to batch add new costume entries for Ultimate Ninja Storm CONNECTIONS

Adds entries to MessageInfo, PlayerSettingParam, CostumeParam. PlayerIcon, CharacterSelectParam, and CostumeBreakParam.

#  Usage
```
cosprm 0.1.0

USAGE:
    cosprm --json <JSON> --dir <DIR>
FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -j, --json <JSON> The path of the .json that will contain the costume config
    -d, --dir <DIR>   The path of your data_win32 directory

ARGS:
    <JSON>
    <DIR>
```

Here is the format of the .json file that's required:
```json
{
    "costumes": [
        {   
            "model_index": 1,
            "characode": "3mnt",
            "modelcode": "nmnc",
            "iconcode": "mnt3",
            "cha_id": "c_cha_999",
            "char_name": "Minato Namikaze (w/o Cloak)",
            "costume_id": "c_costume_011",
            "costume_name": "(w/o Cloak)",
            "color_count": 2,
            "has_costume_break": false
        }
    ]
}
```
