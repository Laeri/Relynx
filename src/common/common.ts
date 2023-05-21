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