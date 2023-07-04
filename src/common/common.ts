import { Collection, RequestTreeNode } from "../bindings";


export const RELYNX_WEBSITE = "https://relynx.app";

type SyntaxHighlightLanguage =
  "html" | "json" | "xml" | "text"

export function getHighlightContentType(contentType: string): SyntaxHighlightLanguage {
  if (contentType === "" || contentType === undefined) {
    return "text"
  }

  if (contentType == "text/html") {
    return "html";
  }
  if (contentType.includes("json")) {
    return "json";
  }
  if (contentType.includes("xml")) {
    return "xml";
  }
  return "text";
}


export function scrollMainToTop() {
  window.scrollTo(0, 0);
  document.querySelector('main')?.scrollTo(0, 0);
}


export function getDefaultGroupName(parent: RequestTreeNode): string {
  let existingNames = parent.children.map((child: RequestTreeNode) => child.name);
  return findFreeDefaultName("New Group", existingNames);
}

export function getDefaultRequestName(parent: RequestTreeNode): string {
  let existingNames = parent.children.map((child: RequestTreeNode) => child.name);
  return findFreeDefaultName("New Request", existingNames);
}

export function getDefaultCollectionName(collections: Collection[]): string {
  let existingNames = collections.map((collection: Collection) => collection.name);
  return findFreeDefaultName("New Collection", existingNames);
}

function findFreeDefaultName(defaultName: string, existingNames: string[]): string {
  let name = defaultName;
  let nameIndex = 1;
  while (existingNames.includes(name)) {
    name = `${defaultName} ${nameIndex}`;
    nameIndex += 1;
  }
  return name;
}

