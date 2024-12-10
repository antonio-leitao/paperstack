let title = $state("Are you sure?");
let message = $state('Loading...');
let show = $state(false);
let permission = $state(null);

export let ConfirmationState = {
    get title() {
        return title;
    },
    set title(val) {
        title = val;
    },
		get message() {
        return message;
    },
    set message(val) {
        message = val;
    },
    get show() {
        return show;
    },
    set show(val) {
        show = val;
    },
		get permission() {
        return permission;
    },
    set permission(val) {
        permission = val;
    },
}

export function promptUserConfirmation(title, message) {
  ConfirmationState.title = title;
  ConfirmationState.message = message;
  ConfirmationState.show = true;

  return new Promise((resolve) => {
    ConfirmationState.permission = resolve;
  });
}

export function confirmAction() {
  ConfirmationState.show = false;
  ConfirmationState.permission(true);
}

export function cancelAction() {
  ConfirmationState.show=false;
  ConfirmationState.permission(false);
}