// TODO fix mime type when serving it
// TODO don't update slider that's being touched by mouse/finger
// TODO throttle

function now() {
  return(Date.now());
}

window.chan_send_delay_ms = 85;
window.last_sent = now();
window.pending_updates = {};

function send_chan_updates() {
  if (now() - window.last_sent < window.chan_send_delay_ms) {
    setTimeout(send_chan_updates, now() - window.last_sent + 1);
    return;
  }

  for (cid in window.pending_updates) {
    var val = window.pending_updates[cid];
    send_chan_value(cid, val);
    delete window.pending_updates[cid];
  }
  window.last_sent = now();
}

function send_chan_value(cid, val) {
  var url = "/chans/" + cid + "/set";
  var body = "value=" + val;
  fetch(url, { method: 'POST', body: body});
  window.last_sent = now();
}

function sliderchange(event) {
  var el = event.srcElement;
  var cid = el.getAttribute("data-channel-id");
  var val = el.value;
  // send_chan_value(cid, val);
  window.pending_updates[cid] = val;
  send_chan_updates();
}
async function fetch_slider_data() {
  return(await fetch('/chans.json').then(response => response.json()));
}

function fetch_update_sliders() {
  var data = fetch_slider_data().then(data => {
    sliders = get_sliders();
    for (s in sliders) {
      var s = sliders[s];
      if (s.getAttribute === undefined) { continue; }
      var cid = s.getAttribute("data-channel-id");

      if (window.touched_sliders[cid] === true) { continue; }

      var val = data[cid];
      s.value = val;
    }
  });
}

function get_sliders() {
  return(document.getElementsByClassName("js-chan-slider"));
}

window.touched_sliders = {};

function touching_slider(event) {
  var el = event.srcElement;
  var cid = el.getAttribute("data-channel-id");
  window.touched_sliders[cid] = true;
}

function untouching_slider(event) {
  var el = event.srcElement;
  var cid = el.getAttribute("data-channel-id");
  window.touched_sliders[cid] = false;
}

window.onload = function onload() {
  var sliders = get_sliders();

  for (let i = 0; i < sliders.length; i++) {
    var slider = sliders[i];
    var cid = slider.getAttribute("data-channel-id");
    window.touched_sliders[cid] = false;

    slider.oninput = sliderchange;
    slider.onmousedown = touching_slider;
    slider.onmouseup = untouching_slider;
  };
  setInterval(fetch_update_sliders, 250);
  setInterval(send_chan_updates, window.chan_send_delay_ms);
  fetch_update_sliders();
}
