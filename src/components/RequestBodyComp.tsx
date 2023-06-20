import {useContext, useState} from "react";
import {BodyType, BodyTypes, newFormProperty, translatedBodyType, updatedRequestModel} from "../models/Request";
import {KeyValueRow} from "./KeyValueRow";
import {useRequestModelStore} from "../stores/requestStore";
import {Dropdown} from "primereact/dropdown";
import {InputTextarea} from "primereact/inputtextarea";
import {Button} from "primereact/button";
import {InputText} from "primereact/inputtext";
import {catchError} from "../common/errorhandling";
import {ToastContext} from "../App";
import {RequestModel, RequestBody} from '../bindings';

interface ComponentProps {
    updateRequest: (newRequest: RequestModel) => void
}

export function RequestBodyComp(props: ComponentProps) {

    const [formProperties, setFormProperties] = useState<FormProperty[]>([new FormProperty()]);

    const requestBody = useRequestModelStore((state) => state.currentRequest?.RequestBody as RequestBody);
    const currentEnvironment = useRequestModelStore((state) => state.currentEnvironment);
    const currentRequest = useRequestModelStore((state) => state.currentRequest as RequestModel);

    const toast = useContext(ToastContext);

    const updateRequestBody = (requestBody: RequestBody, partial: Partial<RequestBody>) => {
        let newRequestBody = updateRequestBody(requestBody, partial);
        let newRequest = updatedRequestModel(currentRequest, {RequestBody: newRequestBody});
        props.updateRequest(newRequest)
    }

    const updateBodyType = (e: any) => {
        updateRequestBody(requestBody, {BodyType: e.target.value as BodyType})
    }

    const updatePlainText = (e: any) => {
        updateRequestBody(requestBody, {PlainText: e.target.value});
    }

    const updateBinaryFilePath = (newPath: string) => {
        updateRequestBody(requestBody, {BinaryFilePath: newPath});
    }

    const updateFormPropertyKey = (formProperty: FormProperty, newKey: string) => {
        let index = formProperties.indexOf(formProperty);
        let newFormProperties = [...formProperties];
        let oldFormProperty = newFormProperties[index];
        let newFormProperty = new FormProperty({
            ...oldFormProperty,
            Key: newKey,
        });
        newFormProperties[index] = newFormProperty;
        setFormProperties(newFormProperties);
        updateRequestBody(requestBody, {FormProperties: newFormProperties});
    }

    const updateFormPropertyValue = (formProperty: FormProperty, newValue: string) => {
        let index = formProperties.indexOf(formProperty);
        let newFormProperties = [...formProperties];
        let oldFormProperty = newFormProperties[index];
        let newFormProperty = new FormProperty({
            ...oldFormProperty,
            Value: newValue
        });
        newFormProperties[index] = newFormProperty;
        setFormProperties(newFormProperties);
        updateRequestBody(requestBody, {FormProperties: newFormProperties})
    }

    const updateFormPropertyActive = (formProperty: FormProperty, active: boolean) => {
        let index = formProperties.indexOf(formProperty);
        let newFormProperties = [...formProperties];
        let oldFormProperty = newFormProperties[index];
        let newFormProperty = new FormProperty({
            ...oldFormProperty,
            Active: active
        });
        newFormProperties[index] = newFormProperty;
        setFormProperties(newFormProperties);
        updateRequestBody(requestBody, {FormProperties: newFormProperties});
    }

    const removeFormProperty = (formProperty: FormProperty) => {
        let newFormProperties = formProperties.filter((current: FormProperty) => current != formProperty);
        setFormProperties(newFormProperties);
        updateRequestBody(requestBody, {FormProperties: newFormProperties});
    }

    const addFormProperty = () => {
        let formProperty = newFormProperty();
        setFormProperties([...formProperties, formProperty]);
        updateRequestBody(requestBody, {FormProperties: [...formProperties, formProperty]});
    }

    const updateJsonData = (newJson: string) => {
        updateRequestBody(requestBody, {JsonData: newJson})
    }

    const selectBinaryFile = () => {
        SelectFile()
            .then((result: string) => {
                updateBinaryFilePath(result);
            })
            .catch(catchError(toast));
    }

    const bodyTypeOptions = Object.entries(BodyTypes).map(([key, value], index: number) => {
        return {name: translatedBodyType(value), value: value}
    });

    return <div style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'flex-start',
        marginBottom: '50px',
    }}>
        <h2 style={{marginBottom: '50px'}}>Request Body</h2>
        <Dropdown optionLabel={"name"} value={requestBody.BodyType} options={bodyTypeOptions} onChange={updateBodyType}
                  style={{marginBottom: '50px'}}/>
        {
            requestBody.BodyType == BodyTypes.NONE &&
            <div>No body</div>
        }
        {requestBody.BodyType == BodyTypes.PLAIN_TEXT &&
            <div style={{display: 'flex', flexDirection: 'column', alignItems: 'flex-start'}}>
                <h4 style={{marginBottom: '20px'}}>Plain Text</h4>
                <InputTextarea value={requestBody.PlainText} onChange={updatePlainText} rows={30}
                               cols={100} autoResize={false} className={'resultText-area'}/>
            </div>
        }
        {
            requestBody.BodyType == BodyTypes.FORM_URL_ENCODED &&
            <div style={{display: 'flex', flexDirection: 'column', alignItems: 'flex-start'}}>
                <h4 style={{marginBottom: '20px'}}>Form Url Encoded</h4>
                {
                    formProperties.map((formProperty: FormProperty, index: number) => {
                        return <KeyValueRow key={index} keyProperty={formProperty.Key}
                                            valueProperty={formProperty.Value}
                                            active={formProperty.Active}
                                            keyLabel={"Name"} valueLabel={"Value"}
                                            updateKey={(key: string) => updateFormPropertyKey(formProperty, key)}
                                            updateValue={(value: string) => updateFormPropertyValue(formProperty, value)}
                                            updateActive={(active: boolean) => updateFormPropertyActive(formProperty, active)}
                                            remove={() => removeFormProperty(formProperty)}
                                            style={{marginTop: '20px'}}
                                            currentEnvironment={currentEnvironment}
                        />
                    })
                }
                <Button icon={'pi pi-plus'} label={"Param"} onClick={addFormProperty} className={"p-button-sm"}
                        style={{marginTop: '40px'}}/>
            </div>
        }
        {
            requestBody.BodyType == BodyTypes.JSON &&
            <div style={{display: 'flex', flexDirection: 'column', alignItems: 'flex-start'}}>
                <h4 style={{marginBottom: '20px'}}>JSON</h4>
                <InputTextarea value={requestBody.JsonData} onChange={(e: any) => updateJsonData(e.target.value)}
                               rows={80}
                               cols={100} autoResize={false} className={'json-body'}/>
            </div>
        }
        {
            requestBody.BodyType == BodyTypes.BINARY_FILE &&
            <div style={{display: 'flex', width: '100%'}}>
                <Button icon={'pi pi-plus'} label={"Choose File"} onClick={selectBinaryFile} className={"p-button-sm"}/>
                <InputText disabled={true} value={requestBody.BinaryFilePath}
                           style={{marginLeft: '20px', flexGrow: 1}}/>
            </div>

        }
    </div>
}
