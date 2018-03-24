/**
 * Long lived background page: https://developer.chrome.com/extensions/background_pages
 *
 * Responsibilities:
 *
 * - monitor onCompleted navigation events
 * - send the event to the weaver background server
 */

const baseUrl = "http://localhost:8464/url";

class NavigationController {
  constructor(webNavigation) {
    webNavigation.onCompleted.addListener(this.onCompletedListener_.bind(this));
  }

  onCompletedListener_(data) {
    const url = data.url;
    if (url.startsWith('http') ) {
      fetch(baseUrl, {
        method: 'POST',
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ url: data.url})
      })
    }
  }
}

let nc = new NavigationController(chrome.webNavigation);
