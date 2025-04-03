import React, { Component, ErrorInfo } from "react";

interface Props {
  /**
   * Optional custom UI to display when an error occurs.
   * If not provided, a default error message will be shown.
   * This allows components to provide their own error handling UI
   * instead of using the generic error message.
   */
  fallback?: React.ReactNode;
  children: React.ReactNode;
}

interface State {
  hasError: boolean;
  fallback?: React.ReactNode;
}

class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, fallback: props.fallback };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("ErrorBoundary caught an error: ", error, errorInfo);
    this.setState({ hasError: true });
  }

  render() {
    if (this.state.hasError) {
      return this.state.fallback || <h1>Something went wrong.</h1>;
    }

    return this.props.children;
  }
}

export default ErrorBoundary;
