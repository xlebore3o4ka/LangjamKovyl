const stdlib: Record<string, (args: string) => string> = {
  "io:print": (args) => `io:format("~s~n", [${args}])`,
};

export default stdlib;
