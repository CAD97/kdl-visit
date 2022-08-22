rg "// [A-Z]+(\(.*?\)|)?:" | lines | parse -r '^(?<path>.*?):.*// (?<keyword>\w*?)(?:\((?P<details>.*?)\))?: (?P<text>.*)$'
