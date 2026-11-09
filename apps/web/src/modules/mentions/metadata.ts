// Cheap list contract: authoritative full-message projection, including aliases.
export function mentionsAny(addresses:readonly string[]|undefined,ownAddresses:readonly string[]):boolean {
 const own=new Set(ownAddresses.map(value=>value.trim().toLowerCase()).filter(value=>value.includes('@')));return addresses?.some(address=>own.has(address.toLowerCase()))??false;
}
