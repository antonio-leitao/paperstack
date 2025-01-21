let dialogState = $state({
  type: "loading",
  title: "",
  message: "",
  isOpen: false,
});

export const DialogStore = {
  get state() {
    return dialogState;
  },

  showLoading(message) {
    dialogState = {
      type: "loading",
      title: "Loading",
      message,
      isOpen: true,
    };
  },

  updateLoading(message) {
    if (dialogState.type === "loading") {
      dialogState.message = message;
    }
  },

  async confirm(title, message) {
    return new Promise((resolve) => {
      dialogState = {
        type: "confirmation",
        title,
        message,
        isOpen: true,
        resolve,
      };
    });
  },

  close() {
    if (dialogState.type === "confirmation" && dialogState.resolve) {
      dialogState.resolve(false);
    }
    dialogState.isOpen = false;
  },

  confirm_action() {
    if (dialogState.type === "confirmation" && dialogState.resolve) {
      dialogState.resolve(true);
    }
    dialogState.isOpen = false;
  },
};
