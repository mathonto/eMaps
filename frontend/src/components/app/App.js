import React from 'react';
import './App.css';
import Osm from "../osm";
import Navigation from "../navigation";
import {toast} from 'react-toastify';

toast.configure();

export default class App extends React.Component {
    constructor(props) {
        super(props);
        this.state = {
            from: {name: undefined, coordinates: undefined},
            to: {name: undefined, coordinates: undefined},
            path: [],
            time: '0h 0min',
            distance: 0
        };

        document.oncontextmenu = () => {
            this.clearMap();
            return false;
        };
    }

    setFrom = (from) => {
        this.setState({
            from: from
        });
    };

    setTo = (to) => {
        this.setState({
            to: to
        });
    };

    setRoute = (path, time, distance) => {
        this.setState({
            path: path,
            time: time,
            distance: distance
        });
    };

    clearMap = () => {
        this.setState({
            from: {name: undefined, coordinates: undefined},
            to: {name: undefined, coordinates: undefined},
            path: [],
            time: '0h 0min',
            distance: 0
        });
    };

    render() {
        return (
            <div>
                <Osm state={this.state}
                     setFrom={this.setFrom}
                     setTo={this.setTo}/>
                <Navigation state={this.state}
                            setRoute={this.setRoute}
                            setFrom={this.setFrom}
                            setTo={this.setTo}
                            clearMap={this.clearMap}/>
            </div>
        );
    }
}