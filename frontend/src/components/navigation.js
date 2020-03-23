import React from "react";
import { Input } from "@material-ui/core";
import Button from "@material-ui/core/Button";
import ButtonGroup from "@material-ui/core/ButtonGroup";
import ToggleButtonGroup from '@material-ui/lab/ToggleButtonGroup';
import ToggleButton from '@material-ui/lab/ToggleButton';
import { DirectionsBike, DirectionsCar } from "@material-ui/icons";
import RadioGroup from "@material-ui/core/RadioGroup";
import FormControlLabel from "@material-ui/core/FormControlLabel";
import Radio from "@material-ui/core/Radio";
import { BASE_URL, NOMINATIM_API, StyledWrapper } from "../config";
import axios from 'axios';
import { toast } from 'react-toastify';
import Divider from '@material-ui/core/Divider';
import Autosuggest from 'react-autosuggest';
import 'react-toastify/dist/ReactToastify.css';
import TextField from '@material-ui/core/TextField';
import LoadingOverlay from 'react-loading-overlay';
import styled from 'styled-components'
import EvStationIcon from '@material-ui/icons/EvStation';


const StyledLoader = styled(LoadingOverlay)`
  width: 372px;
  height: 342px;
  .MyLoader_overlay {
    background: rgba(232, 236, 241, 1));
  }
  &.MyLoader_wrapper--active {
    overflow: hidden;
  }
`

const getSuggestionValue = suggestion => suggestion.properties.display_name;

/**
 * Only render suggestion if user input is at least 3 chars.
 * @param value: user input
 * @returns {boolean}
 */
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
            max_range: '',
            isCalculating: false,
            clickedChargingShow: false
        };

        document.oncontextmenu = () => {
            this.reset();
            return false;
        };
    }

    /**
     * This function renders the html code for the suggestion.
     * @param suggestion: name of the suggestion
     * @returns {*}
     */
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

    /**
     * Called when user changes input.
     * @param event: event of change
     * @param newValue: changed value
     */
    onChange = (event, {newValue}) => {
        this.setState({
            value: newValue
        });
    };

    /**
     * Called when suggestions need to be fetched from nominatim api.
     * @param value: input value of user
     */
    onSuggestionsFetchRequested = ({value}) => {
        axios.post(NOMINATIM_API + '/search/?q=' + value + '&format=geojson&countrycodes=de').then(response => response.data)
            .then(data => this.setState({suggestions: data.features.slice(0, 7)}));
    };

    /**
     * Called when user clears input.
     */
    onSuggestionsClearRequested = () => {
        this.setState({
            suggestions: []
        });
    };

    /**
     * Called when user selects a suggestion.
     */
    onSuggestionSelected = () => {
        this.setState({
            value: ''
        });
    };

    onShowingChargingStations = () => {
        this.setState({
            clickedChargingShow: !this.state.clickedChargingShow
        });
    }

    /**
     * Called when calculation is started to display loading screen.
     */
    onCalculating = () => {
        this.setState({
            isCalculating: !this.state.isCalculating
        });
    };

    /**
     * Called when user selects a suggestion as start.
     * @param suggestion: selected suggestion
     */
    asStart = (suggestion) => {
        // get name and coordinates of suggestion
        const name = suggestion.properties.display_name;
        const coordinates = suggestion.geometry.coordinates.reverse();
        this.props.setFrom({
            name: name,
            coordinates: coordinates
        });
    };

    /**
     * Called when user selects a suggestion as destination.
     * @param suggestion: selected suggestion
     */
    asDest = (suggestion) => {
        // get name and coordinates of suggestion
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
            <StyledLoader
                active={this.state.isCalculating}
                spinner
                classNamePrefix='MyLoader_'
                text='Calculating route...'
            >
                <div id='navigation' style={{width: '350px'}}>
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
                            {this.chargingButtonRenderer()}
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
            </StyledLoader>
        )
            ;
    }

    chargingButtonRenderer() {
        return (
            this.state.clickedChargingShow ? <Button
                id='charging-stations'
                variant="contained"
                onClick={this.showChargingStations}>HIDE<EvStationIcon></EvStationIcon></Button> : <Button
                id='charging-stations'
                variant="contained"
                onClick={this.showChargingStations}>SHOW<EvStationIcon></EvStationIcon></Button>
        )
    }

    showChargingStations = () => {
        const chargingMarkers = [];
        if (this.state.clickedChargingShow) {
            this.props.setAllChargingStations([]);
            this.onShowingChargingStations();
            return;

        }
        axios.get(BASE_URL + '/charging-stations').then(response => {
                // extract visited charging stations
                for (const charging_coords of response.data.charging_coords) {
                    chargingMarkers.push(Object.values(charging_coords));
                }
                this.props.setAllChargingStations(chargingMarkers);
                this.onShowingChargingStations();
            }
        ).catch(err => {
            toast.error(err.response.data);
        });
    }

    /**
     * Reset everything.
     */
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

    /**
     * Start shortest path calculation.
     */
    go = () => {
        // check if everything required for route calculation is set
        if (!this.props.state.from.coordinates || !this.props.state.to.coordinates
            || !this.state.current_range || !this.state.max_range) {
            toast.error('Please select start, goal, current and max. range');
            return;
            // sanity check for input of current and max. range
        } else if (Number(this.state.current_range) > Number(this.state.max_range)) {
            toast.error('Current range cannot be bigger than max. range');
            return;
        }
        this.onCalculating();
        // extract data from state
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
        // shortest path request with data
        axios.post(BASE_URL + '/shortest-path', data).then(response => {
            this.onCalculating();
            const path = [];
            const visited_charging_stations = [];
            // extract path
            for (const coordinates of response.data.path) {
                path.push(Object.values(coordinates));
            }
            // extract visited charging stations
            for (const charging_coords of response.data.visited_charging_coords) {
                visited_charging_stations.push(Object.values(charging_coords));
            }
            // set route
            this.props.setRoute(
                path,
                this.hhmm(response.data.time),
                this.round(response.data.distance / 1000),
                visited_charging_stations
            );
        }).catch(err => {
            toast.error(err.response.data);
            this.onCalculating();
        });
    };

    /**
     * Round distance.
     * @param value: distance of route
     * @returns {number}
     */
    round = (value) => {
        return Math.round(value * 10) / 10
    };

    /**
     * Format time needed for route.
     * @param secs: time in second
     * @returns {string}
     */
    hhmm = (secs) => {
        // extract hours and minutes
        const hours = Math.floor(secs / 3600);
        const minutes = Math.floor((secs - (hours * 3600)) / 60);
        return hours + 'h ' + minutes + 'min';
    };

    /**
     * Setter for navigation type (car/bike).
     * @param event: event called when toggle is clicked
     * @param newNavType: updated value
     */
    handleNavType = (event, newNavType) => {
        this.setState({
            transport: newNavType,
        });
    };

    /**
     * Setter for routing preference (time/distance).
     * @param event: event called when toggle is clicked
     */
    handleMetric = (event) => {
        this.setState({
            routing: event.target.value
        });
    };

    /**
     * Called when input for current range changes.
     * @param e: changed input event
     */
    currentRangeChange = (e) => {
        // check if current range is bigger than maximum range
        if (this.state.max_range.length > 0 && Number(e.target.value) > Number(this.state.max_range)) {
            toast.error('Current range cannot be bigger than maximum range');
        } else {
            if (this.sanitizeRangeInput(e.target.value)) {
                this.setState({current_range: e.target.value})
            } else {
                toast.error('Please enter a number (below the max. value of 4294968)');
            }
        }
    }

    /**
     * Sanitize user input for range.
     * @param range: input of user
     * @returns {boolean}
     */
    sanitizeRangeInput = (range) => {
        // regex to sanitize input of range
        const re = /^[0-9\b]+$/;

        if (range === '' || re.test(range)) {
            if ((Number(range) < 4294968)) {
                return true;
            }
        }
        return false;
    }

    /**
     * Called when input for max range changes.
     * @param e: changed input event
     */
    maxRangeChange = (e) => {
        if (this.sanitizeRangeInput(e.target.value)) {
            this.setState({max_range: e.target.value})
        } else {
            toast.error('Please enter a number (below the max. value of 4294968)');
        }
    }
}
