//will be adding more utils fucntions here
export const objectToJsonString = obj => {
    BigInt.prototype.toJSON = function() { return this.toString() }
    return JSON.stringify(obj)
}
