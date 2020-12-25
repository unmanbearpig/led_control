#include <libusb.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <arpa/inet.h>
#include "../common/protocol.h"
#include "../common/secrets.h"

#define PWM_PERIOD 22126

void print_hex_bytes(char *buf, int len) {
  for (int i = 0; i < len; i++) {
    printf("%02x", buf[i]);
  }
  printf("\n");
}

int is_our_device(struct libusb_device_descriptor *desc) {
  if (desc->idVendor != 0xCAFE) {
    return 0;
  }

  if (desc->idProduct != 0xCAFE) {
    return 0;
  }

  return 1;
}

char *endpoint_transfer_type_str(uint8_t endp_bmAttributes) {
  switch(endp_bmAttributes) {
  case LIBUSB_TRANSFER_TYPE_CONTROL:
    return "Control";
  case LIBUSB_TRANSFER_TYPE_ISOCHRONOUS:
    return "Isochronous";
	case LIBUSB_TRANSFER_TYPE_BULK:
    return "Bulk";
	case LIBUSB_TRANSFER_TYPE_INTERRUPT:
    return "Interrupt";
	case LIBUSB_TRANSFER_TYPE_BULK_STREAM:
    return "Bulk Stream";
  }

  return "Unkown Transfer Type";
}

char *format_err(int err) {
  if (err == LIBUSB_ERROR_NO_MEM) {
    return("LIBUSB_ERROR_NO_MEM");
  }

  if (err == LIBUSB_ERROR_ACCESS) {
    return("LIBUSB_ERROR_ACCESS");
  }

  if (err == LIBUSB_ERROR_NO_DEVICE) {
    return("LIBUSB_ERROR_NO_DEVICE");
  }

  if (err == LIBUSB_ERROR_TIMEOUT) {
    return("LIBUSB_ERROR_TIMEOUT");
  }

  if (err == LIBUSB_ERROR_PIPE) {
    return("LIBUSB_ERROR_PIPE");
  }

  if (err == LIBUSB_ERROR_OVERFLOW) {
    return("LIBUSB_ERROR_OVERFLOW");
  }

  if (err == LIBUSB_ERROR_NO_DEVICE) {
    return("LIBUSB_ERROR_NO_DEVICE");
  }

  if (err == LIBUSB_ERROR_BUSY) {
    return("LIBUSB_ERROR_BUSY");
  }

  if (err == LIBUSB_ERROR_INVALID_PARAM) {
    return("LIBUSB_ERROR_INVALID_PARAM");
  }

  if (err == 0) {
    return("Success");
  }

  return("Unknown error");
}

// transparent function that calls libusb_open and logs errors if any
int open_device(libusb_device *dev, libusb_device_handle **dev_handle) {
  int err = libusb_open(dev, dev_handle);
  if (err == 0) {
    return err;
  }

  if (err == LIBUSB_ERROR_NO_MEM) {
    fprintf(stderr, "libusb_open: LIBUSB_ERROR_NO_MEM\n");
    return err;
  }

  if (err == LIBUSB_ERROR_ACCESS) {
    fprintf(stderr, "libusb_open: LIBUSB_ERROR_ACCESS\n");
    return err;
  }

  if (err == LIBUSB_ERROR_NO_DEVICE) {
    fprintf(stderr, "libusb_open: LIBUSB_ERROR_NO_DEVICE\n");
    return err;
  }

  fprintf(stderr, "libusb_open: unknown error %d\n", err);
  return err;
}

// returns 0 if no errors and the out_dev_handle has been filled
//         anything else otherwise
int find_device(libusb_context *usb_ctx, libusb_device_handle **out_dev_handle) {
  libusb_device **dev_list = NULL;
  libusb_device *found_dev = NULL;
  int err = 0;
  ssize_t cnt = libusb_get_device_list(usb_ctx, &dev_list);

  if (cnt <= 0) {
    fprintf(stderr, "find_device error: found %ld devices\n", cnt);
    exit(1);
    return 1;
  }

  struct libusb_device_descriptor desc = { 0 };
  for (int i = 0; i < cnt; i++) {
    fflush(stdout);
    err = libusb_get_device_descriptor(dev_list[i], &desc);
    if (err != 0) { // should never happen according to the docs
      fprintf(stderr, "could not get device descriptor\n");
      return err;
    }

    if (is_our_device(&desc)) {
      found_dev = dev_list[i];
      break;
    }
  }

  int handle_opened = 0;
  if (found_dev != NULL) {
    err = open_device(found_dev, out_dev_handle);
    if (err == 0) {
      handle_opened = 1;
    }
  }

  // not sure, but we probably should unreference the devices
  //   (second argument)
  libusb_free_device_list(dev_list, 1);

  if (!handle_opened) {
    return 1;
  }
  return 0;
}

char *get_transfer_error_str(int err) {
  switch(err) {
  case LIBUSB_ERROR_TIMEOUT:
    return "timeout";
  case LIBUSB_ERROR_PIPE:
    return "pipe error";
  case LIBUSB_ERROR_OVERFLOW:
    return "overflow error";
  case LIBUSB_ERROR_NO_DEVICE:
    return "no device";
  case LIBUSB_ERROR_BUSY:
    return "busy";
  case LIBUSB_ERROR_INVALID_PARAM:
    return "invalid param";
  case 0:
    return "success";
  default:
    return "other error";
  }
}

struct usb {
  libusb_context *ctx;
  struct libusb_config_descriptor *config;
  libusb_device_handle *dev_handle;
};

/* returns error */
int setup_usb(struct usb *usb) {
  usb->ctx = NULL;

  int r = libusb_init(&usb->ctx);
  if (r < 0) {
    fprintf(stderr, "libusb error: %d\n", r);
    exit(1);
  }

  usb->dev_handle = NULL;
  int err = find_device(usb->ctx, &usb->dev_handle);
  if (err != 0) {
    fprintf(stderr, "could not find the device\n");
    exit(1);
  }

  libusb_device *dev = libusb_get_device(usb->dev_handle);
  struct libusb_device_descriptor desc = { 0 };
  err = libusb_get_device_descriptor(dev, &desc);
  if (err != 0) {
    fprintf(stderr, "libusb_get_device_descriptor error %d\n", err);
    exit(1);
  }
  if (desc.bNumConfigurations != 1) {
    fprintf(stderr, "found %d configurations, expected 1\n", desc.bNumConfigurations);
    exit(1);
  }

  int bConfigurationValue = 0;
  err = libusb_get_configuration(usb->dev_handle, &bConfigurationValue);
  if (err != 0) {
    fprintf(stderr, "libusb_get_configuration: %d\n", err);
    exit(1);
  }

  if (bConfigurationValue != 1) {
    fprintf(stderr, "strage, I expected bConfigurationValue to be 1 but it's %d\n",
            bConfigurationValue);
  }

  usb->config = NULL;
  err = libusb_get_config_descriptor_by_value(dev, bConfigurationValue, &usb->config);
  if (err != 0) {
    fprintf(stderr, "libusb_get_config_descriptor_by_value: %d\n", err);
    exit(1);
  }

  printf("has %d interfaces\n", usb->config->bNumInterfaces);

  for (int i = 0; i < usb->config->bNumInterfaces; i++) {
    const struct libusb_interface *interface = &usb->config->interface[i];
    printf("Interface %d: has %d altsettings\n", i, interface->num_altsetting);

    for (int alt_idx = 0; alt_idx < interface->num_altsetting; alt_idx++) {
      const struct libusb_interface_descriptor *altsetting = &interface->altsetting[alt_idx];
      printf("  %d %d endpoints=%d\n", i, alt_idx, altsetting->bNumEndpoints);

      for (int endp_idx = 0; endp_idx < altsetting->bNumEndpoints; endp_idx++) {
        const struct libusb_endpoint_descriptor *endpoint = &altsetting->endpoint[endp_idx];

        printf("    endp %d %.2x DT=%.2x %s\n",
               endp_idx,
               endpoint->bEndpointAddress,
               endpoint->bDescriptorType,
               endpoint_transfer_type_str(endpoint->bmAttributes));
      }
    }
  }

  int interrupt_if_id = 2;
  err = libusb_claim_interface(usb->dev_handle, interrupt_if_id);
  if (err != 0) {
    fprintf(stderr, "could not claim interrupt interface: %s\n", format_err(err));
    exit(1);
  }

  /* int bulk_if_id = 1; */
  /* err = libusb_claim_interface(dev_handle, bulk_if_id); */
  /* if (err != 0) { */
  /*   fprintf(stderr, "could not claim bulk interface: %s\n", format_err(err)); */
  /*   exit(1); */
  /* } */

  /* char cdc_out_str[] = "hello, device!"; */
  /* unsigned int cdc_transfered = 0; */
  /* err = libusb_bulk_transfer(dev_handle, 0x01, cdc_out_str, sizeof(cdc_out_str), &cdc_transfered, 60); */
  /* if (err != 0) { */
  /*   fprintf(stderr, "send: %d: %s\n", err, get_transfer_error_str(err)); */
  /* } */
}

void free_and_close_usb(struct usb *usb) {
  libusb_free_config_descriptor(usb->config);
  libusb_close(usb->dev_handle);
  libusb_exit(usb->ctx);
}

void print_hex(char *bytes, size_t len) {
  for(size_t i = 0; i < len; i++) {
    fprintf(stderr, "%x ", bytes[i]);
  }
  fprintf(stderr, "\n");
}

int main(int argc, char **argv) {
  struct usb usb = { 0, 0, 0 };
  int err = setup_usb(&usb);
  if (err != 0) {
    exit(1);
  }

  /* udp setup */
  int sock = 0;

  if ((sock = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP)) == -1)	{
    fprintf(stderr, "socket failed\n");
    exit(1);
	}

  struct sockaddr_in sin;
  memset(&sin, 0, sizeof(sin));

  sin.sin_family = AF_INET;
  sin.sin_addr.s_addr = htonl(INADDR_ANY);
  sin.sin_port = htons(DEFAULT_UDP_PORT);

  if (bind(sock, (struct sockaddr *)&sin, sizeof(sin)) == -1) {
    fprintf(stderr, "bind failed\n");
    exit(1);
  }

  LedValuesMessage msg;
  memset(&msg, 0, sizeof(msg));

  struct sockaddr_storage peer_addr;
  unsigned int peer_addr_len = 0;
  memset(&peer_addr, 0, sizeof(peer_addr));

  LedValuesMessage recv_msg;
  memset(&recv_msg, 0xEE, sizeof(recv_msg));

  uint16_t usb_msg[3] = { 0, 0, 0 };
  int sent_bytes = 0;
  int usb_transfer_timeout = 60;

  for (;;) {
    int recsize = recvfrom(sock, &msg, sizeof(msg), 0, (struct sockaddr *)&peer_addr, &peer_addr_len);
    fprintf(stderr, "got packet; size: %d\n", recsize);

    if (recsize != -1) {
      char printbuf[512];
      int printed = 0;

      printf("hello");
      printed += sprintf(printbuf, "bytes:");
      for (int i = 0; i < recsize; i++) {
        printed += sprintf(printbuf + printed -1, " %02x", ((uint8_t *)&msg)[i]);
      }
      printbuf[printed] = 0;
      fprintf(stderr, "%s\n", printbuf);

      if (sock < 0) {
        fprintf(stderr, "accept failed\n");
        exit(1);
      }

      if (msg.magic != LED_VALUES_MESSAGE_MAGIC) {
        fprintf(stderr, "wrong magic\n");
        continue;
      }

      if (msg.type & LED_WRITE == 0 || msg.type & LED_CONFIG != 0) {
        fprintf(stderr, "wrong msg type\n");
        continue;
      }

      LedMsgData *data = &msg.payload.data;
      fprintf(stderr, "flags = %x amount = %f\n", data->flags, data->amount);

      if (data->flags & LED_VALUES_FLAG_FLOAT != 0) {
        fprintf(stderr, "float\n");
        // convert float to uint16_t
        usb_msg[0] = data->values.values_float[0] * PWM_PERIOD;
        usb_msg[1] = data->values.values_float[1] * PWM_PERIOD;
        usb_msg[2] = data->values.values_float[2] * PWM_PERIOD;
      } else {
        fprintf(stderr, "u16\n");
        usb_msg[0] = data->values.values16[0];
        usb_msg[1] = data->values.values16[1];
        usb_msg[2] = data->values.values16[2];
      }

      // todo print hex byte values of floats
      fprintf(stderr, "vals 16: %x %x %x %x (last ignored)\n",
              data->values.values16[0], data->values.values16[1],
              data->values.values16[2], data->values.values16[3]);
      fprintf(stderr, "vals 32: %f %f %f %f (last ignored)\n",
              data->values.values_float[0], data->values.values_float[1],
              data->values.values_float[2], data->values.values_float[3]);

      fprintf(stderr, "f1: ");
      print_hex((char *)&data->values.values_float[0], sizeof(data->values.values_float[0]));
      fprintf(stderr, "f2: ");
      print_hex((char *)&data->values.values_float[1], sizeof(data->values.values_float[1]));
      fprintf(stderr, "f3: ");
      print_hex((char *)&data->values.values_float[2], sizeof(data->values.values_float[2]));
      fprintf(stderr, "f4: ");
      print_hex((char *)&data->values.values_float[3], sizeof(data->values.values_float[3]));

      fprintf(stderr, "values : %x %x %x %x (last ignored)\n", usb_msg[0], usb_msg[1], usb_msg[2], usb_msg[3]);
      fprintf(stderr, "usb_msg: %x %x %x\n", usb_msg[0], usb_msg[1], usb_msg[2]);

      err = libusb_interrupt_transfer(usb.dev_handle,
                                      0x05,
                                      (unsigned char *)usb_msg, sizeof(usb_msg),
                                      &sent_bytes,
                                      usb_transfer_timeout);

      if (err != 0) {
        fprintf(stderr, "send: %d: %s\n", err, get_transfer_error_str(err));
      }

      /* sendto(sock, &recv_msg, sizeof(recv_msg), 0, (struct sockaddr *)&peer_addr, peer_addr_len); */
    }
  }


  /* char data[128] = { 0 }; */
  /* memset(data, 'X', sizeof(data)); */
  /* int recv_bytes = 0; */
  /* int sent_bytes = 0; */
  /* int transfer_timeout = 60; */
  /* while(1) { */
  /*   err = libusb_interrupt_transfer(usb.dev_handle, */
  /*                                   0x86, */
  /*                                   data, sizeof(data), */
  /*                                   &recv_bytes, */
  /*                                   transfer_timeout); */

  /*   if (err != 0) { */
  /*     fprintf(stderr, "recv: %d: %s\n", err, get_transfer_error_str(err)); */
  /*   } */

  /*   printf("received %d bytes: ", recv_bytes); */
  /*   for(int i = 0; i < recv_bytes; i++) { */
  /*     putchar(data[i]); */
  /*   } */
  /*   printf("\n"); */

  /*   print_hex_bytes(data, recv_bytes); */

  /*   err = libusb_interrupt_transfer(usb.dev_handle, */
  /*                                   0x05, */
  /*                                   data, sizeof(data), */
  /*                                   &recv_bytes, */
  /*                                   transfer_timeout); */
  /*   if (err != 0) { */
  /*     fprintf(stderr, "send: %d: %s\n", err, get_transfer_error_str(err)); */
  /*   } */
  /* } */

  free_and_close_usb(&usb);
  return 0;
}
