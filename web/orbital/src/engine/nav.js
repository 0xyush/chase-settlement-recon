// Sidebar clicks: scroll to the top of the page, or to the results tabs once they have rendered.
export const scrollToView = (target) =>
  setTimeout(() => {
    if (target === "top") document.getElementById("page-scroll")?.scrollTo({ top: 0, behavior: "smooth" });
    else document.getElementById(target)?.scrollIntoView({ behavior: "smooth", block: "start" });
  }, 60);
