export type AnyError = {
    description: string,
    debugFormat: string,
};

export type IoError = AnyError & {
    kind: string,
    rawOsError: number | null
};

export type ErrorInfo = {
    type: "getBaseDirs"
} | {
    type: "unhandledReleaseFileType",
    releaseType: string
} | {
    type: "convertPathToStringError",
    path: string
} | {
    type: "extractSetlistPath" | "extractZipError" | "verifyFail" | "downloadFail",
    error: AnyError,
} | {
    type: "failedToRevealFolder" | "invalidSignatureFile",
    path: string,
    error: AnyError
} | {
    type: "failedToRecreateFolder" | "createYARCDirectory" | "createLauncherDirectory" | "createTempDirectory" | "createSetlistDirectory" | "extractFileOpenError" | "writeTagFileError" | "verifyOpenZipFail" | "downloadFileCreateFail" | "failedToRemoveTagFile",
    path: string,
    error: IoError,
} | {
    type: "downloadInitFail",
    url: string,
    error: AnyError
} | {
    type: "downloadWriteError",
    path: string,
    url: string,
    error: IoError
} | {
    type: "failedToLaunchProfile",
    path: string,
    arguments: string[],
    useObsVkapture: boolean,
    error: IoError
};