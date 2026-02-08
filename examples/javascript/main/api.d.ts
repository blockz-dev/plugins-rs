declare namespace tools {
    function colors(enabled: boolean): void;
}

declare namespace filesystem {
    function read(path : string): string;
    function write(path : string, content : string): void;
    function remove(path : string): void;
}