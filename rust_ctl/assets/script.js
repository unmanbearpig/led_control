// TODO fix mime type when serving it
// TODO add js for buttons

function sliderchange(event) {
  var el = event.srcElement;
  var form = el.closest('form');

  var form_data = new FormData(form);

  var req_vals = [];
  var entries = form_data.entries();
  for (var pair of form_data.entries()) {
    req_vals.push(encodeURIComponent(pair[0]) + "=" + encodeURIComponent(pair[1]));
  }
  var req_body = req_vals.join("&");

  fetch(form.action, {
    method: form.method,
    body: req_body,
  });
}

window.onload = function onload() {
  var sliders = document.getElementsByClassName("js-chan-slider");

  for (let i = 0; i < sliders.length; i++) {
    var slider = sliders[i];

    slider.oninput = sliderchange;
  };
}
