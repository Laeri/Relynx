import { ToastContext } from "../App";
import { FrontendError } from "../model/error";

// @TODO
export function catchError(toast: ToastContext): (err: FrontendError) => void {
  return (err: FrontendError) => {
    console.error('catch error: ', err)
    toast.showError('', err.message ?? "");
  }
}

export type FError = FrontendError | undefined

export function displayAndLogErr(error: FrontendError, toast: ToastContext) {
  //@TODO: use log plugin functions
  /* LogFrontendError(error).then(() => {
    // ignored
  }).catch((err: string) => {
    console.error("Could not log frontend error correctly in displayAndLogErr!: ", err);
  }) */
  //console.error(`Error id: ${error.Id}, title: ${error.Title}, userMsg: ${error.UserMsg}, errorMsg: ${error.Error}`);
  //toast.showError(error.Title, error.UserMsg);
  if (error.message) {
    toast.showError("", error.message ?? "");
  }
}

export function formatParseErrorsMsg(parseErrors: FrontendError[]): string {
  /*@TODO let result = "Folowing errors occurred: \n\n";

  result += parseErrors.map((parseError: FrontendError) => {
    return `${parseError.Item}:${parseError.Path} Request file is malformed -> ${parseError.Description} \n`
  }); */

  //return result;
  return ""
}
