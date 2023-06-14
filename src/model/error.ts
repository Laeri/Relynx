import { FrontendError } from '../bindings';

export type BackendError = string

export function newFrontendError(): FrontendError {
  return {
    kind: "Generic",
    message: ""
  }
}

export function NewFError(id: string, title: string, userMsg: string, errorMsg: string): FrontendError {
  let frontendError = newFrontendError()
  /*@TODO:frontendError.Title = title;
  frontendError.Id = id;
  frontendError.UserMsg = userMsg;
  frontendError.Error = errorMsg;*/
  return frontendError
}

export type CancellationToken = {
  cancelled: boolean
}
