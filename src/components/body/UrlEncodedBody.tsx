import { RequestBodyUrlEncoded } from "../model/request"

interface ComponentProps {
  body: RequestBodyUrlEncoded
}

export function UrlEncodedBody(props: ComponentProps) {
  return (
    <div>UrlEncodedBody</div>
  )
}
