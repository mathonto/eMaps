import React from "react";
import {Input} from "@material-ui/core";
import Button from "@material-ui/core/Button";
import ButtonGroup from "@material-ui/core/ButtonGroup";
import ToggleButtonGroup from '@material-ui/lab/ToggleButtonGroup';
import ToggleButton from '@material-ui/lab/ToggleButton';
import {DirectionsBike, DirectionsCar} from "@material-ui/icons";
import RadioGroup from "@material-ui/core/RadioGroup";
import FormControlLabel from "@material-ui/core/FormControlLabel";
import Radio from "@material-ui/core/Radio";
import {BASE_URL, NOMINATIM_API, StyledWrapper} from "../config";
import axios from 'axios';
import {toast} from 'react-toastify';
import Divider from '@material-ui/core/Divider';
import Autosuggest from 'react-autosuggest';
import 'react-toastify/dist/ReactToastify.css';
import TextField from '@material-ui/core/TextField';

const getSuggestionValue = suggestion => suggestion.properties.display_name;

function shouldRenderSuggestions(value) {
    return value.trim().length > 2;
}

export default class Navigation extends React.Component {

    constructor(props) {
        super(props);
        this.state = {
            transport: "car",
            routing: "time",
            value: '',
            suggestions: [],
            current_range: '',
            max_range: ''
        };

        document.oncontextmenu = () => {
            this.reset();
            return false;
        };
    }

    renderSuggestion = suggestion => (
        <div>
            <Divider/>
            <div style={{marginTop: "5px", marginBottom: "5px"}}>
                {suggestion.properties.display_name}
            </div>
            <div>
                <ButtonGroup size="small" aria-label="small outlined button group">
                    <Button id="start" onClick={() => this.asStart(suggestion)}>SET AS START</Button>
                    <Button id="dest" onClick={() => this.asDest(suggestion)}>SET AS DESTINATION</Button>
                </ButtonGroup>
            </div>
        </div>
    );


    onChange = (event, {newValue}) => {
        this.setState({
            value: newValue
        });
    };

    onSuggestionsFetchRequested = ({value}) => {
        axios.post(NOMINATIM_API + '/search/?q=' + value + '&format=geojson&countrycodes=de').then(response => response.data)
            .then(data => this.setState({suggestions: data.features.slice(0, 7)}));
    };

    onSuggestionsClearRequested = () => {
        this.setState({
            suggestions: []
        });
    };

    onSuggestionSelected = () => {
        this.setState({
            value: ''
        });
    };

    asStart = (suggestion) => {
        const name = suggestion.properties.display_name;
        const coordinates = suggestion.geometry.coordinates.reverse();
        this.props.setFrom({
            name: name,
            coordinates: coordinates
        });
    };

    asDest = (suggestion) => {
        const name = suggestion.properties.display_name;
        const coordinates = suggestion.geometry.coordinates.reverse();
        this.props.setTo({
            name: name,
            coordinates: coordinates
        });
    };

    render() {
        const {value, suggestions} = this.state;
        const inputProps = {
            placeholder: 'Type a location',
            value,
            onChange: this.onChange
        };

        return (
            <div id='navigation'>
                <StyledWrapper>
                    <Autosuggest
                        suggestions={suggestions}
                        onSuggestionsFetchRequested={this.onSuggestionsFetchRequested}
                        onSuggestionsClearRequested={this.onSuggestionsClearRequested}
                        getSuggestionValue={getSuggestionValue}
                        renderSuggestion={this.renderSuggestion}
                        shouldRenderSuggestions={shouldRenderSuggestions}
                        onSuggestionSelected={this.onSuggestionSelected}
                        inputProps={inputProps}
                    />
                </StyledWrapper>
                <Input id='input-from'
                       placeholder='From'
                       value={this.props.state.from.name || ''}/>
                <Input id='input-to'
                       placeholder='To'
                       value={this.props.state.to.name || ''}/>

                <div id="nav-settings">
                    <ToggleButtonGroup id="button-group"
                                       value={this.state.transport}
                                       exclusive
                                       onChange={this.handleNavType}>
                        <ToggleButton value="car">
                            <DirectionsCar/>
                        </ToggleButton>
                        <ToggleButton value="bike">
                            <DirectionsBike/>
                        </ToggleButton>
                    </ToggleButtonGroup>

                    <RadioGroup value={this.state.routing}
                                onChange={this.handleMetric}>
                        <div id="nav-metric-radios">
                            <FormControlLabel
                                value="time"
                                control={<Radio color="primary"/>}
                                label="Time"
                                labelPlacement="start"/>
                            <FormControlLabel
                                value="distance"
                                control={<Radio color="primary"/>}
                                label="Distance"
                                labelPlacement="start"/>
                        </div>
                    </RadioGroup>
                </div>
                <div style={{marginTop: '10px'}}>
                    <form className='rowC' noValidate autoComplete="off">
                        <TextField id="outlined-basic" label="Current range (km)"
                                   value={this.state.current_range}
                                   onChange={this.currentRangeChange}/>
                                   <div style={{width: '15px'}}></div>
                        <TextField id="outlined-basic-2" label="Max. range (km)"
                                   value={this.state.max_range}
                                   onChange={this.maxRangeChange}/>
                    </form>
                </div>
                <div>
                    <ButtonGroup fullWidth aria-label="split button">
                        <Button
                            id='go'
                            variant="contained"
                            onClick={this.go}>GO</Button>
                        <Button id='reset'
                                variant="contained"
                                onClick={this.reset}
                        >RESET</Button>
                    </ButtonGroup>
                </div>
                <div id="travel">
                    <a>{this.props.state.time} | </a>
                    <a>{this.props.state.distance}km</a>
                </div>
            </div>
        )
            ;
    }

    reset = () => {
        this.props.clearMap();
        this.setState({
            transport: "car",
            routing: "time",
            value: "",
            suggestions: [],
            current_range: '',
            max_range: ''
        })
    };

    go = () => {
        if (!this.props.state.from.coordinates || !this.props.state.to.coordinates
            || !this.state.current_range || !this.state.max_range) {
            toast.error('Please select start, goal, current and max. range');
            return;
        } else if (Number(this.state.current_range) > Number(this.state.max_range)) {
            toast.error('Current range cannot be bigger than max. range');
            return;
        }
        const data = {
            start: {
                lat: this.props.state.from.coordinates[0],
                lon: this.props.state.from.coordinates[1],
            },
            goal: {
                lat: this.props.state.to.coordinates[0],
                lon: this.props.state.to.coordinates[1],
            },
            transport: this.state.transport,
            routing: this.state.routing,
            current_range: this.state.current_range,
            max_range: this.state.max_range
        };

        axios.post(BASE_URL + '/shortest-path', data).then(response => {
            console.log(response);
            const path = [];
            const visited_charging_stations = [];
            for (const coordinates of response.data.path) {
                path.push(Object.values(coordinates));
            }
            for (const charging_coords of response.data.visited_charging_coords) {
                visited_charging_stations.push(Object.values(charging_coords));
            }
            this.props.setRoute(
                path,
                this.hhmm(response.data.time),
                this.round(response.data.distance / 1000),
                visited_charging_stations
            );
        }).catch(err => toast.error(err.response.data));
    };

    round = (value) => {
        return Math.round(value * 10) / 10
    };

    hhmm = (secs) => {
        const hours = Math.floor(secs / 3600);
        const minutes = Math.floor((secs - (hours * 3600)) / 60);
        return hours + 'h ' + minutes + 'min';
    };

    handleNavType = (event, newNavType) => {
        this.setState({
            transport: newNavType,
        });
    };

    handleMetric = (event) => {
        this.setState({
            routing: event.target.value
        });
    };

    currentRangeChange = (e) => {
        const re = /^[0-9\b]+$/;

        if (this.state.max_range.length > 0 && Number(e.target.value) > Number(this.state.max_range)) {
            toast.error('Current range cannot be bigger than maximum range');
        } else {
            if (e.target.value === '' || re.test(e.target.value)) {
                this.setState({current_range: e.target.value})
            } else {
                toast.error('Please enter a number');
            }
        }
    }

    maxRangeChange = (e) => {
        const re = /^[0-9\b]+$/;

        if (e.target.value === '' || re.test(e.target.value)) {
            this.setState({max_range: e.target.value})
        } else {
            toast.error('Please enter a number');
        }
    }
}
