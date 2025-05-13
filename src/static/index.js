(function() {
  htmx.config.scrollBehavior = 'auto'

  // Function to copy message link to clipboard
  window.copyMessageLink = function(messageId) {
    const url = `${window.location.origin}/message/${messageId}`;
    navigator.clipboard.writeText(url).then(() => {
      // Visual feedback could be added here
    });
  };

  // State for infinite scroll (upwards) and general scroll container
  let scrollContainer = null;
  let originalScrollHeight = 0;
  let originalScrollTop = 0;
  let isProcessingOlderMessagesLoad = false;

  // Helper to get the scroll container, initializing it once
  function getScrollContainer() {
    if (!scrollContainer) {
      scrollContainer = document.querySelector('#content');
    }
    return scrollContainer;
  }

  document.body.addEventListener('htmx:configRequest', function(evt) {
    const currentScrollContainer = getScrollContainer();
    if (!currentScrollContainer) return;

    // Check if this request is for loading older messages (direction=up)
    // and triggered by intersection (typical for scrollers).
    if (evt.detail.path.includes('?direction=up') &&
        evt.detail.triggeringEvent && evt.detail.triggeringEvent.type === 'intersect') {
      originalScrollHeight = currentScrollContainer.scrollHeight;
      originalScrollTop = currentScrollContainer.scrollTop;
      isProcessingOlderMessagesLoad = true;
    }
  });

  document.body.addEventListener('htmx:afterSwap', function(evt) {
    const currentScrollContainer = getScrollContainer();

    // Infinite scroll adjustment for upward loading (older messages)
    if (isProcessingOlderMessagesLoad && currentScrollContainer &&
        evt.detail.requestConfig.path && evt.detail.requestConfig.path.includes('?direction=up')) {
      const newScrollHeight = currentScrollContainer.scrollHeight;
      const addedHeight = newScrollHeight - originalScrollHeight;

      if (addedHeight > 0) { // Ensure content was actually added
        currentScrollContainer.scrollTop = originalScrollTop + addedHeight;
      }
      isProcessingOlderMessagesLoad = false;
    }

    // Update channel list if the response includes a channel ID header,
    // and the request wasn't for loading the channel list itself.
    const headerChannelId = evt.detail.xhr.getResponseHeader('X-Current-Channel-Id');
    const channelsElement = document.getElementById('channels');

    if (headerChannelId && channelsElement) {
      const requestPath = evt.detail.requestConfig.path;
      // Check if the request that just completed was for loading/reloading the channel list itself.
      const isChannelListUpdateRequest = requestPath.startsWith('/channels');

      if (!isChannelListUpdateRequest) {
        channelsElement.setAttribute('hx-get', `/channels?current_channel_id=${headerChannelId}`);
        htmx.process(channelsElement);
        htmx.trigger(channelsElement, 'load', { isChannelUpdate: true });
      }
    }
  });

  document.body.addEventListener('htmx:afterSettle', function(evt) {
    const currentScrollContainer = getScrollContainer();
    if (!currentScrollContainer) return;

    const targetMessage = document.getElementById('target-message');

    if (targetMessage && !targetMessage.hasAttribute('data-scrolled') && currentScrollContainer.contains(targetMessage)) {
      // If a target message exists, is not yet scrolled to, and is within our scroll container
      targetMessage.scrollIntoView({ behavior: 'smooth', block: 'start' });
      targetMessage.setAttribute('data-scrolled', 'true'); // Mark as scrolled to prevent re-scrolling the same DOM element
    } else if (!targetMessage && evt.detail.target && evt.detail.target.id === 'content') {
      const requestUrl = evt.detail.xhr.responseURL || (evt.detail.requestConfig && evt.detail.requestConfig.path);
      if (requestUrl) {
        const channelPageMatch = requestUrl.match(/\/channel\/\d+\/(\d+)/);
        if (channelPageMatch) {
          const pageNum = parseInt(channelPageMatch[1], 10);
          if (pageNum === 0 && !requestUrl.includes('direction=') && !(evt.detail.requestConfig && evt.detail.requestConfig.triggeringEvent && evt.detail.requestConfig.triggeringEvent.detail && evt.detail.requestConfig.triggeringEvent.detail.isChannelUpdate)) {
            currentScrollContainer.scrollTop = currentScrollContainer.scrollHeight; // Scroll to bottom
          }
        }
      }
    }
  });
}());
