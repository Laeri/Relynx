import { ExternalToast, ToastContext } from "../App";
import { FrontendError } from "../model/error";

export function catchError(err: any) {
  console.error('catch error: ', err)
  if (ExternalToast) {
    ExternalToast.showError('', err.message ?? "An error occurred");
  }
}

export function catchErrorWithTitle(title: string, err: any) {
  console.error('catch error: ', err)
  if (ExternalToast) {
    ExternalToast.showError(title, err.message ?? "An error occurred");
  }
}

export type FError = FrontendError | undefined

export function displayAndLogErr(error: FrontendError, toast: ToastContext) {
  console.error("Frontend error occurred: ", error);
  toast.showError(error.title, error.message);
}

