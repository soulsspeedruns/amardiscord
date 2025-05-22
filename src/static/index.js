(() => {
  htmx.config.scrollBehavior = "auto";

  window.copyMessageLink = (messageId) => {
    const url = `${window.location.origin}/message/${messageId}`;
    navigator.clipboard.writeText(url);
  };

  let scrollContainer = null;
  let originalScrollHeight = 0;
  let originalScrollTop = 0;
  let isProcessingOlderMessagesLoad = false;

  function getScrollContainer() {
    if (!scrollContainer) {
      scrollContainer = document.querySelector("#content");
    }
    return scrollContainer;
  }

  document.body.addEventListener("htmx:configRequest", (evt) => {
    const currentScrollContainer = getScrollContainer();
    if (!currentScrollContainer) return;

    if (
      evt.detail.path.includes("?direction=up") &&
      evt.detail.triggeringEvent?.type === "intersect"
    ) {
      originalScrollHeight = currentScrollContainer.scrollHeight;
      originalScrollTop = currentScrollContainer.scrollTop;
      isProcessingOlderMessagesLoad = true;
    }
  });

  document.body.addEventListener("htmx:afterSwap", (evt) => {
    const currentScrollContainer = getScrollContainer();

    if (
      isProcessingOlderMessagesLoad &&
      currentScrollContainer &&
      evt.detail.requestConfig.path?.includes("?direction=up")
    ) {
      const newScrollHeight = currentScrollContainer.scrollHeight;
      const addedHeight = newScrollHeight - originalScrollHeight;

      if (addedHeight > 0) {
        currentScrollContainer.scrollTop = originalScrollTop + addedHeight;
      }
      isProcessingOlderMessagesLoad = false;
    }

    const headerChannelId = evt.detail.xhr.getResponseHeader(
      "X-Current-Channel-Id",
    );
    const channelsElement = document.getElementById("channels");

    if (headerChannelId && channelsElement) {
      const requestPath = evt.detail.requestConfig.path;
      const isChannelListUpdateRequest = requestPath.startsWith("/channels");

      if (!isChannelListUpdateRequest) {
        channelsElement.setAttribute(
          "hx-get",
          `/channels?current_channel_id=${headerChannelId}`,
        );
        htmx.process(channelsElement);
        htmx.trigger(channelsElement, "load", { isChannelUpdate: true });
      }
    }
  });

  document.body.addEventListener("htmx:afterSettle", (evt) => {
    const currentScrollContainer = getScrollContainer();
    if (!currentScrollContainer) return;

    const targetMessage = document.getElementById("target-message");

    if (
      targetMessage &&
      !targetMessage.hasAttribute("data-scrolled") &&
      currentScrollContainer.contains(targetMessage)
    ) {
      targetMessage.scrollIntoView({ behavior: "smooth", block: "start" });
      targetMessage.setAttribute("data-scrolled", "true");
    } else if (!targetMessage && evt.detail.target?.id === "content") {
      const requestUrl =
        evt.detail.xhr.responseURL || evt.detail.requestConfig?.path;
      if (requestUrl) {
        const channelPageMatch = requestUrl.match(/\/channel\/\d+\/(\d+)/);
        if (channelPageMatch) {
          const pageNum = Number.parseInt(channelPageMatch[1], 10);
          if (
            pageNum === 0 &&
            !requestUrl.includes("direction=") &&
            !evt.detail.requestConfig?.triggeringEvent?.detail?.isChannelUpdate
          ) {
            currentScrollContainer.scrollTop =
              currentScrollContainer.scrollHeight;
          }
        }
      }
    }
  });
})();
