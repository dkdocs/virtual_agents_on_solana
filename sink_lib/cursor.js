import fs from "fs";
import { Labels } from "./constants.js";
import { getTokenCursor, getTransferCursor, updateTokenCursor, updateTransferCursor } from "./clickhouseWrapper.js";
// import { getTokenCursor, updateCursor } from "./clickhouseWrapper.js";

export const getCursor = async (substreamPackage) => {
  const CURSOR_FILE = getCursorFileName(substreamPackage);
  try {
    return await fs.promises.readFile(CURSOR_FILE);
    // return await getTokenCursor(substreamPackage)
    // if(substreamPackage === Labels.tokensSubstreamPackage) {
    //   return await getTokenCursor();
    // } else {
    //   return await getTransferCursor();
    // }
  } catch (e) {
    return undefined;
  }
};

// In this example, the cursor is persisted in a file.
export const writeCursor = async (cursor, substreamPackage, blockNumber) => {
  const CURSOR_FILE = getCursorFileName(substreamPackage);
  try {
    await fs.promises.writeFile(CURSOR_FILE, cursor);
    // await updateCursor(substreamPackage, cursor);
    // if(substreamPackage === Labels.tokensSubstreamPackage) {
    //   await updateTokenCursor(cursor, blockNumber)
    // } else {
    //   await updateTransferCursor(cursor, blockNumber)
    // }
  } catch (e) {
    throw new Error("COULD_NOT_COMMIT_CURSOR");
  }
};

const getCursorFileName = (substreamPackage) => {
  const filename = substreamPackage === "tokenPackage" ? "./tokenCursor" : substreamPackage === "transferPackage" ? "./transferCursor" : null;
  return filename;
};
