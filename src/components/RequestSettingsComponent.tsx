import { Button } from "primereact/button"
import { Checkbox } from "primereact/checkbox"
import { InputText } from "primereact/inputtext"
import { useState } from "react"
import { Collection, RequestModel, RequestSettings } from "../bindings"
import { catchError } from "../common/errorhandling"
import { updatedRequestModel } from "../model/model"
import { backend } from "../rpc"
import { HelpTooltip } from "./HelpTooltip"

interface ComponentProps {
  request: RequestModel
  updateRequest: (newRequest: RequestModel) => void
  collection: Collection
}

export function RequestSettingsComponent(props: ComponentProps) {

  const [responseFilepathValid, setResponseFilepathValid] = useState<boolean>(true);

  const updateRequestSettings = (partial: Partial<RequestSettings>) => {
    let new_settings = { ...props.request.settings, ...partial };
    let request = updatedRequestModel(props.request, { settings: new_settings });
    props.updateRequest(request);
  }

  const updateSaveResponse = (shouldSaveToFile: boolean) => {
    console.log('should save: ', shouldSaveToFile);
    let newRequest = updatedRequestModel(props.request, {});
    newRequest.redirect_response.save_response = shouldSaveToFile;
    props.updateRequest(newRequest);
  }

  const updateResponseFilepath = (path: string) => {
    let newRequest = updatedRequestModel(props.request, {});
    newRequest.redirect_response.save_path = path;
    props.updateRequest(newRequest);

    backend.validateResponseFilepath(path).then((valid: boolean) => {
      setResponseFilepathValid(valid);
    }).catch(catchError);
  }

  const updateOverwriteResponse = (overwrite: boolean) => {
    let newRequest = updatedRequestModel(props.request, {});
    newRequest.redirect_response.overwrite = overwrite;
    props.updateRequest(newRequest);

  }

  const selectResponseFilePath = () => {
    backend.getResponseFilepath(props.request.rest_file_path).then((result: string) => {
      let newRequest = updatedRequestModel(props.request, {});
      newRequest.redirect_response.save_path = result;
      props.updateRequest(newRequest);
    }).catch(catchError);
  }
  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
      <h2 style={{ marginBottom: '20px' }}>RequestSettings</h2>
      {/* When request sending is implemented, check if all of these are needed and disable and mark those that are not present*/}
      <div style={{ display: 'flex', alignItems: 'center' }}>
        <Checkbox inputId="no_redirect" name="no_redirect" value="no_redirect" onChange={(e) => updateRequestSettings({ no_redirect: e.target.checked })} checked={props.request.settings.no_redirect ?? false} />

        <div style={{ display: 'flex', justifyContent: 'space-between', width: '200px' }}>
          <label htmlFor="no_redirect" style={{ marginLeft: '20px' }}>No redirect</label>
          <HelpTooltip style={{ marginLeft: '20px' }} text="Determines when sending a request if we should follow the redirect or not. If active no follow is done and the result is directly returned." />
        </div>
      </div>

      <div style={{ marginTop: '10px', display: 'flex', alignItems: 'center' }}>
        <Checkbox inputId="no_cookie_jar" name="no_cookie_jar" value="no_cookie_jar" onChange={(e) => updateRequestSettings({ no_cookie_jar: e.target.checked })} checked={props.request.settings.no_cookie_jar ?? false} />

        <div style={{ display: 'flex', justifyContent: 'space-between', width: '200px' }}>
          <label htmlFor="no_cookie_jar" style={{ marginLeft: '20px' }}>No cookie jar</label>
          <HelpTooltip style={{ marginLeft: '20px' }} text="Prevents saving any received cookies within the http-client.cookies jar so you do not have to remove them manually everytime if you don't want them." />
        </div>
      </div>

      <div style={{ marginTop: '10px', display: 'flex', alignItems: 'center' }}>
        <Checkbox inputId="no_log" name="no_log" value="no_log" onChange={(e) => updateRequestSettings({ no_log: e.target.checked })} checked={props.request.settings.no_log ?? false} />
        <div style={{ display: 'flex', justifyContent: 'space-between', width: '200px' }}>
          <label htmlFor="no_log" style={{ marginLeft: '20px' }}>No log</label>
          <HelpTooltip style={{ marginLeft: '20px' }} text="With this setting you can prevent that any response received of this request is saved in the history. Use this if the response contains sensitive data" />
        </div>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'start', marginTop: '30px' }}>
        <h3>Save Response</h3>
        <p style={{ marginTop: '20px', textAlign:'left' }}>Choose if you want to save the response of a request to a file</p>
        <div style={{ display: 'flex', alignItems: 'center', marginTop: '20px' }}>
          <Checkbox inputId="save_response" name="save_response" value="save_response" onChange={(e) => updateSaveResponse(e.target.checked ?? false)} checked={props.request.redirect_response.save_response} />
          <label htmlFor="save_response" style={{ marginLeft: '20px' }}>Save response to file</label>
          <HelpTooltip style={{ marginLeft: '20px' }} text="If checked the response from a request will be saved to a chosen file." />
        </div>

        {props.request.redirect_response.save_response &&
          <div style={{ marginTop: '30px', display: 'flex', flexDirection: 'column', alignItems: 'start' }}>

            <p style={{ textAlign: 'start' }}>Choose a path where you want to save the response. Choose a relative path that is the same for all members of your team. Consider choosing a folder or file extension that you can gitignore.</p>

            <div style={{ marginTop: '20px', display: 'flex', alignItems: 'center' }}>
              <InputText disabled={true} aria-invalid="true" style={{ flexGrow: 1 }} value={props.request.redirect_response.save_path ?? ''} onChange={(e) => { updateResponseFilepath(e.target.value) }} />
              <Button label={"Select new path"}
                onClick={selectResponseFilePath} style={{ marginLeft: '30px' }} />
            </div>
            {!responseFilepathValid &&
              <div className="p-error" style={{ marginTop: '20px' }}>The folder where the response should be saved doesn't exist!</div>
            }

            <div style={{ display: 'flex', alignItems: 'center', marginTop: '20px' }}>
              <Checkbox inputId="overwrite_file" name="overwrite_file" value="overwrite_file" onChange={(e) => updateOverwriteResponse(e.target.checked ?? false)} checked={props.request.redirect_response.overwrite} />
              <label htmlFor="overwrite_file" style={{ marginLeft: '20px' }}>Overwrite File </label>
              <HelpTooltip style={{ marginLeft: '20px' }} text="If overwrite is chosen then the file will be overwriten for every new response received. If not checked a new file will be generated with appended number -1, -2, ..." />
            </div>
          </div>
        }
      </div>
    </div>

  )
}
