function exitWithError(message: string, error?: unknown, exitCode: number = 1): never {
  console.error(message, error);
  process.exit(exitCode);
}

export default exitWithError;
