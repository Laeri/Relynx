import { Accordion, AccordionTab } from 'primereact/accordion';
import { ProgressSpinner } from 'primereact/progressspinner';
import { RequestResult } from '../bindings';
import { SendRequestButton } from './SendRequestButton';
import { SingleRequestResult } from "./SingleRequestResult";

interface ComponentProps {
  requestResult?: RequestResult,
  resultHistory: RequestResult[],
  isSendingRequest: boolean,
  requestSendDisabled: boolean,
  clearResult: (result: RequestResult) => void,
  sendRequest: () => void,
  cancelRequest: () => void
}

export function ResultDisplay(props: ComponentProps) {

  return (
    <div className={"resultText-container"}
      style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', marginTop: '10px', marginBottom: '30px' }}>

      <SendRequestButton style={{marginBottom: '20px'}} overwriteLabel={props.requestResult !== undefined ? "Resend" : "Send"} disabled={props.requestSendDisabled} doRequest={props.sendRequest} cancelRequest={props.cancelRequest} isSendingRequest={props.isSendingRequest} />

      <div style={{ width: '100%' }}>
        {
          props.isSendingRequest && <ProgressSpinner style={{ maxHeight: '80px' }} />
        }
        <div>
          {
            (props.requestResult === undefined && props.resultHistory.length === 0 && !props.isSendingRequest) &&
            <div className="fade-in-fast" style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
              <h2>No Request Has Been Sent Yet</h2>
              <p style={{ marginTop: '20px' }}>
                There are no request results present. Send a request to get a new result.
              </p>
            </div>
          }

          {

            (!props.isSendingRequest && props.requestResult) &&
            <SingleRequestResult requestResult={props.requestResult} clearResult={props.clearResult} />
          }

          {
            /* first element is already shown at the top, if no current request then it is not within the history and we can display it*/
            (props.resultHistory.length > 0) &&
            <div style={{ marginTop: '30px', display: 'flex', flexDirection: 'column', alignItems: 'flex-start', width: '100%' }}>
              <h3>History</h3>
              <div style={{ marginTop: '20px', width: '100%' }}>
                <Accordion style={{ width: '100%', marginBottom: '30px' }}>
                  {props.resultHistory.map((result: RequestResult, index: number) => {
                    return <AccordionTab header={`Result ${(props.resultHistory.length - index)}`} style={{ width: '100%' }}>
                      <SingleRequestResult requestResult={result} clearResult={props.clearResult} key={index} />
                    </AccordionTab>
                  })}
                </Accordion>
              </div>
            </div>
          }
        </div>
      </div>
    </div>
  )
}
