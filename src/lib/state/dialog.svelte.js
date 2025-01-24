const DialogActions = {
  CANCEL: 'cancel',
  REPLACE: 'replace',
  KEEP: 'keep',
  UPDATE: 'update'
};

let dialogState = $state({
  type: "loading",
  title: "",
  message: "",
  isOpen: false,
  data: null,
  resolve: null
});

export const DialogStore = {
  get state() {
    return dialogState;
  },

  start(message) {
    dialogState = {
      type: "loading",
      title: "Loading",
      message,
      isOpen: true,
      data: null,
      resolve: null
    };
  },

  lap(message) {
    if (dialogState.type === "loading") {
      dialogState = {
        ...dialogState,
        message
      };
    }
  },

  async confirm(title, message) {
    return new Promise((resolve) => {
      dialogState = {
        type: "confirmation",
        title,
        message,
        isOpen: true,
        data: null,
        resolve
      };
    });
  },

  async handleDuplicate(paper, duplicatePaper) {
    return new Promise((resolve) => {
      dialogState = {
        type: "duplicate",
        title: "Duplicate Entry",
        message: `A paper with similar details was found: "${duplicatePaper.bib.title}"`,
        isOpen: true,
        data: { paper, duplicatePaper },
        resolve
      };
    });
  },

  close() {
    if (dialogState.resolve) {
      dialogState.resolve(DialogActions.CANCEL);
    }
    dialogState = {
      ...dialogState,
      isOpen: false
    };
  },

  confirm_action() {
    if (dialogState.resolve) {
      dialogState.resolve(true);
    }
    dialogState = {
      ...dialogState,
      isOpen: false
    };
  },

  selectAction(action) {
    if (dialogState.resolve) {
      dialogState.resolve(action);
    }
    dialogState = {
      ...dialogState,
      isOpen: false
    };
  }
};

export { DialogActions };
